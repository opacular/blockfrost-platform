use super::connection::NodeClient;
use crate::{
    cbor::haskell_types::{TxSubmitFail, TxValidationError},
    BlockfrostError,
};
use pallas_crypto::hash::Hasher;
use pallas_network::miniprotocols::{
    localstate,
    localtxsubmission::{EraTx, Response},
};
use tracing::{info, warn};

impl NodeClient {
    /// Submits a transaction to the connected Cardano node.
    pub async fn submit_transaction(&mut self, tx: String) -> Result<String, BlockfrostError> {
        let tx = hex::decode(tx).map_err(|e| BlockfrostError::custom_400(e.to_string()))?;
        let txid = hex::encode(Hasher::<256>::hash_cbor(&tx));

        let current_era = self
            .with_statequery(|generic_client: &mut localstate::GenericClient| {
                Box::pin(async {
                    Ok(localstate::queries_v16::get_current_era(generic_client).await?)
                })
            })
            .await?;
        let era_tx = EraTx(current_era, tx);

        // Connect to the node
        let submission_client = self.client.as_mut().unwrap().submission();

        // Submit the transaction
        match submission_client.submit_tx(era_tx).await {
            Ok(Response::Accepted) => {
                info!("Transaction accepted by the node {}", txid);
                Ok(txid)
            }
            Ok(Response::Rejected(reason)) => {
                let reason = reason.0;
                let msg_res = Self::try_decode_error(&reason);

                match msg_res {
                    Ok(decoded_error) => {
                        let error_response = Self::generate_error_response(decoded_error);
                        let error_message = serde_json::to_string(&error_response)
                            .unwrap_or_else(|_| "Failed to serialize error response".to_string());
                        warn!(error_message);

                        Err(BlockfrostError::tx_submit(error_message))
                    }

                    Err(e) => {
                        warn!("Failed to decode error reason: {:?}", e);

                        Err(BlockfrostError::tx_submit(format!(
                            "Failed to decode error reason: {:?}",
                            e
                        )))
                    }
                }
            }
            Err(e) => {
                let error_message = format!("Error during transaction submission: {:?}", e);

                Err(BlockfrostError::tx_submit(error_message))
            }
        }
    }

    /// Mimicks the data structure of the error response from the cardano-submit-api
    fn generate_error_response(error: TxValidationError) -> TxSubmitFail {
        use crate::cbor::haskell_types::{
            TxCmdError::TxCmdTxSubmitValidationError, TxSubmitFail::TxSubmitFail,
            TxValidationErrorInCardanoMode::TxValidationErrorInCardanoMode,
        };

        TxSubmitFail(TxCmdTxSubmitValidationError(
            TxValidationErrorInCardanoMode(error),
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::cbor::haskell_types::{
        ApplyConwayTxPredError::*, ApplyTxErr, ShelleyBasedEra::*, TxValidationError::*,
    };

    use super::*;

    #[test]
    fn test_generate_error_response_with_multiple_errors() {
        let validation_error = ShelleyTxValidationError {
            error: ApplyTxErr(vec![
                MempoolFailure("error1".to_string()),
                MempoolFailure("error2".to_string()),
            ]),
            era: ShelleyBasedEraConway,
        };

        let error_string =
            serde_json::to_string(&NodeClient::generate_error_response(validation_error))
                .expect("Failed to convert error to JSON");
        let expected_error_string = r#"{"tag":"TxSubmitFail","contents":{"tag":"TxCmdTxSubmitValidationError","contents":{"tag":"TxValidationErrorInCardanoMode","contents":{"kind":"ShelleyTxValidationError","error":["MempoolFailure (error1)","MempoolFailure (error2)"],"era":"ShelleyBasedEraConway"}}}}"#;

        assert_eq!(error_string, expected_error_string);
    }
}
