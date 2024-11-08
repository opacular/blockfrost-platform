#![allow(dead_code)]

use pallas::ledger::addresses::StakeKeyHash;
use pallas_codec::minicbor;
use pallas_codec::minicbor::Decode;
use pallas_codec::utils::Bytes;
use pallas_primitives::{
    byron::{TxIn, TxOut},
    conway::{Coin, DatumHash, ExUnits, RewardAccount, ScriptHash, VKeyWitness, Value},
};

/// This file contains the types that are mapped from the Haskell codebase.
/// Type examples:
/// https://github.com/IntersectMBO/ouroboros-consensus/blob/82c5ebf7c9f902b7250144445f45083c1c13929e/ouroboros-consensus-cardano/src/shelley/Ouroboros/Consensus/Shelley/Eras.hs#L334
/// https://github.com/IntersectMBO/cardano-node-emulator/blob/ba5c4910a958bbccb38399f6a871459e46701a93/cardano-node-emulator/src/Cardano/Node/Emulator/Internal/Node/Validation.hs#L255
/// https://github.com/IntersectMBO/cardano-node/blob/master/cardano-testnet/test/cardano-testnet-test/files/golden/tx.failed.response.json.golden
///
/// Haskell references to the types are commented next to them.
/// Here are some more type referernces:
/// https://github.com/IntersectMBO/cardano-ledger/blob/78b20b6301b2703aa1fe1806ae3c129846708a10/libs/cardano-ledger-core/src/Cardano/Ledger/BaseTypes.hs#L737
/// https://github.com/IntersectMBO/cardano-ledger/blob/master/eras/mary/impl/src/Cardano/Ledger/Mary/Value.hs
/// https://github.com/IntersectMBO/cardano-ledger/blob/master/libs/cardano-ledger-core/src/Cardano/Ledger/Coin.hs

// https://github.com/IntersectMBO/cardano-api/blob/a0df586e3a14b98ae4771a192c09391dacb44564/cardano-api/internal/Cardano/Api/Eon/ShelleyBasedEra.hs#L271
#[derive(Debug)]
pub enum ShelleyBasedEra {
    Shelley(),
    Allegra(),
    Mary(),
    Alonzo(),
    Babbage(),
    Conway(),
}

#[derive(Debug)]
pub struct ApplyTxErr(pub Vec<ApplyConwayTxPredError>);

// https://github.com/IntersectMBO/cardano-ledger/blob/aed1dc28b98c25ea73bc692e7e6c6d3a22381ff5/eras/conway/impl/src/Cardano/Ledger/Conway/Rules/Ledger.hs#L146
#[derive(Debug)]
pub enum ApplyConwayTxPredError {
    UtxowFailure(ConwayUtxoWPredFailure),
    CertsFailure(ConwayUtxoWPredFailure),
    GovFailure(ConwayUtxoWPredFailure),
    WdrlNotDelegatedToDRep(StakeKeyHash),
    TreasuryValueMismatch(Coin),
    TxRefScriptsSizeTooBig(u64),
    MempoolFailure(String),
}

// https://github.com/IntersectMBO/cardano-ledger/blob/f54489071f4faa4b6209e1ba5288507c824cca50/eras/conway/impl/src/Cardano/Ledger/Conway/Rules/Utxow.hs
#[derive(Debug)]
pub enum ConwayUtxoWPredFailure {
    UtxoFailure(ConwayUtxoPredFailure),
    InvalidWitnessesUTXOW(VKeyWitness),
    MissingVKeyWitnessesUTXOW(VKeyWitness),
    MissingScriptWitnessesUTXOW(ScriptHash),
    ScriptWitnessNotValidatingUTXOW(ScriptHash),
    MissingTxBodyMetadataHash(Bytes),      // auxDataHash
    MissingTxMetadata(Bytes),              // auxDataHash
    ConflictingMetadataHash(Bytes, Bytes), // Mismatch auxDataHash
    InvalidMetadata(),                     // empty
    ExtraneousScriptWitnessesUTXOW(ScriptHash),
    MissingRedeemers(Vec<(PlutusPurpose, ScriptHash)>),
    MissingRequiredDatums(Vec<DatumHash>, Vec<DatumHash>), // set of missing data hashes, set of recieved data hashes
    NotAllowedSupplementalDatums(Vec<DatumHash>, Vec<DatumHash>), // set of unallowed data hashes, set of acceptable data hashes
    PPViewHashesDontMatch(Option<ScriptIntegrityHash>),
    UnspendableUTxONoDatumHash(Vec<TxIn>), //  Set of transaction inputs that are TwoPhase scripts, and should have a DataHash but don't
    ExtraRedeemers(Vec<PlutusPurpose>),    // List of redeemers not needed
    MalformedScriptWitnesses(Vec<ScriptHash>),
    MalformedReferenceScripts(Vec<ScriptHash>),
}

// https://github.com/IntersectMBO/cardano-ledger/blob/f54489071f4faa4b6209e1ba5288507c824cca50/eras/conway/impl/src/Cardano/Ledger/Conway/Rules/Utxo.hs#L315
#[derive(Debug)]
pub enum ConwayUtxoPredFailure {
    UtxosFailure(Box<ConwayUtxoPredFailure>),
    BadInputsUTxO(Vec<TxIn>),
    OutsideValidityIntervalUTxO(ValidityInterval, SlotNo), // validity interval, current slot
    MaxTxSizeUTxO(u64),                                    // less than or equal
    InputSetEmptyUTxO(),                                   // empty
    FeeTooSmallUTxO(Coin, Coin),                           // Mismatch expected, supplied
    ValueNotConservedUTxO(Value, Value),
    WrongNetwork(Network, Vec<Addr>), // the expected network id,  the set of addresses with incorrect network IDs
    WrongNetworkWithdrawal(Network, Vec<RewardAccount>), // the expected network id ,  the set of reward addresses with incorrect network IDs
    OutputTooSmallUTxO(Vec<TxOut>),
    OutputBootAddrAttrsTooBig(Vec<TxOut>),
    OutputTooBigUTxO(Vec<(u64, u64, TxOut)>), //  list of supplied bad transaction output triples (actualSize,PParameterMaxValue,TxOut)
    InsufficientCollateral(Coin, Coin), // balance computed, the required collateral for the given fee
    ScriptsNotPaidUTxO(Utxo),           // The UTxO entries which have the wrong kind of script
    ExUnitsTooBigUTxO(ExUnits),         // check: The values are serialised in reverse order
    CollateralContainsNonADA(Value),
    WrongNetworkInTxBody(), // take in Network, https://github.com/IntersectMBO/cardano-ledger/blob/78b20b6301b2703aa1fe1806ae3c129846708a10/libs/cardano-ledger-core/src/Cardano/Ledger/BaseTypes.hs#L779
    OutsideForecast(SlotNo),
    TooManyCollateralInputs(u64), // this is Haskell Natural, how many bit is it?
    NoCollateralInputs(),         // empty
    IncorrectTotalCollateralField(Coin, Coin), // collateral provided, collateral amount declared in transaction body
    BabbageOutputTooSmallUTxO(Vec<(TxOut, Coin)>), // list of supplied transaction outputs that are too small, together with the minimum value for the given output
    BabbageNonDisjointRefInputs(Vec<TxIn>), // TxIns that appear in both inputs and reference inputs
}

// wrapping  TxValidationError (ShelleyTxValidationError or ByronTxValidationError) in TxValidationErrorInCardanoMode
// https://github.com/IntersectMBO/cardano-api/blob/a0df586e3a14b98ae4771a192c09391dacb44564/cardano-api/internal/Cardano/Api/InMode.hs#L289
// https://github.com/IntersectMBO/cardano-api/blob/a0df586e3a14b98ae4771a192c09391dacb44564/cardano-api/internal/Cardano/Api/InMode.hs#L204
// toJson https://github.com/IntersectMBO/cardano-api/blob/a0df586e3a14b98ae4771a192c09391dacb44564/cardano-api/internal/Cardano/Api/InMode.hs#L233
#[derive(Debug)]
pub enum TxValidationError {
    Byron(ApplyTxErr),
    Shelley(ApplyTxErr, ShelleyBasedEra),
}

// https://github.com/IntersectMBO/cardano-ledger/blob/f54489071f4faa4b6209e1ba5288507c824cca50/libs/cardano-ledger-core/src/Cardano/Ledger/Address.hs
// the bytes are not decoded
pub type Addr = Bytes;

// https://github.com/IntersectMBO/cardano-ledger/blob/78b20b6301b2703aa1fe1806ae3c129846708a10/eras/alonzo/impl/src/Cardano/Ledger/Alonzo/Scripts.hs#L497
// not tested yet
#[derive(Debug)]
pub enum PlutusPurpose {
    Spending,   // 0
    Minting,    // 1
    Certifying, // 2
    Rewarding,  // 3
}
// https://github.com/IntersectMBO/cardano-ledger/blob/78b20b6301b2703aa1fe1806ae3c129846708a10/libs/cardano-ledger-core/src/Cardano/Ledger/BaseTypes.hs#L779
#[derive(Debug, Decode)]
pub enum Network {
    #[n(0)]
    Mainnet,
    #[n(1)]
    Testnet,
}
// https://github.com/IntersectMBO/cardano-ledger/blob/aed1dc28b98c25ea73bc692e7e6c6d3a22381ff5/eras/alonzo/impl/src/Cardano/Ledger/Alonzo/TxBody/Internal.hs#L162
// not tested yet
type ScriptIntegrityHash = ScriptHash;

// https://github.com/IntersectMBO/cardano-ledger/blob/aed1dc28b98c25ea73bc692e7e6c6d3a22381ff5/eras/allegra/impl/src/Cardano/Ledger/Allegra/Scripts.hs#L109
#[derive(Debug, Decode)]
pub struct ValidityInterval {
    #[n(0)]
    pub invalid_before: Option<SlotNo>, // SlotNo
    #[n(1)]
    pub invalid_hereafter: Option<SlotNo>, // SlotNo
}

// https://github.com/IntersectMBO/cardano-ledger/blob/aed1dc28b98c25ea73bc692e7e6c6d3a22381ff5/libs/cardano-ledger-core/src/Cardano/Ledger/UTxO.hs#L83
#[derive(Debug)]
pub struct Utxo(pub Vec<(TxIn, TxOut)>);

type SlotNo = u64;
