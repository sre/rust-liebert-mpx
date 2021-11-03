// Liebert MPX PDU Rust API
// © 2021 Sebastian Reichel
// SPDX-License-Identifier: ISC

#![crate_type = "lib"]
#![crate_name = "liebert_mpx"]

//! This Rust crate can be used to access information from
//! Liebert MPX PDUs (power distribution units) by using
//! its web interface.
//!
//! # Examples
//! ```no_run
//! extern crate liebert_mpx as liebert;
//!
//! /// Print information about currently present events/alarms
//! fn main() {
//!     let pdu = liebert::MPX::new("192.168.23.42", "Liebert", "Liebert");
//!     async {
//!         let events = pdu.get_events().await.unwrap();
//!         for event in events {
//!             println!("{:?}", event);
//!         }
//!     };
//! }
//! ```
//!
//! ```no_run
//! extern crate liebert_mpx as liebert;
//!
//! /// List receptacles and their status
//! fn main() {
//!     let pdu = liebert::MPX::new("192.168.23.42", "Liebert", "Liebert");
//!     async {
//!         let receptacles = pdu.get_receptacles().await.unwrap();
//!         for receptacle in receptacles {
//!             println!("{:?}", receptacle);
//!         }
//!     };
//! }
//! ```
//!
//! ```no_run
//! extern crate liebert_mpx as liebert;
//!
//! /// Set receptacle label
//! fn main() {
//!     let pdu = liebert::MPX::new("192.168.23.42", "Liebert", "Liebert");
//!     async {
//!         let receptacle = pdu.get_info_receptacle(1, 2, 3).await.unwrap();
//!         let settings = liebert::ReceptacleSettings {
//!             label: "Low Power Light".to_string(),
//!             ..receptacle.settings
//!         };
//!         pdu.set_receptacle_settings(1, 2, 3, &settings).await.unwrap();
//!     };
//! }
//! ```
//!
//! ```no_run
//! extern crate liebert_mpx as liebert;
//!
//! /// Send commands to PDU1, Branch 1, Receptacle 1-4
//! fn main() {
//!     let pdu = liebert::MPX::new("192.168.23.42", "Liebert", "Liebert");
//!     async {
//!         pdu.receptacle_identify(1, 1, 1).await.unwrap();
//!         pdu.receptacle_disable(1, 1, 2).await.unwrap();
//!         pdu.receptacle_enable(1, 1, 3).await.unwrap();
//!         pdu.receptacle_reboot(1, 1, 4).await.unwrap();
//!     };
//! }
//! ```

use serde::Serialize;
use std::collections::HashMap;
use std::str::FromStr;

type RawDataTable = HashMap<String, TableValue>;
pub type EnumParseError = ();
pub type EventList = Vec<Event>;
pub type ReceptacleList = Vec<ReceptacleListEntry>;

#[derive(Debug, Clone)]
/// Parsing Error - PDU did not provide required information
pub struct MissingDataError;

impl std::fmt::Display for MissingDataError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "could not find required data")
    }
}

impl std::error::Error for MissingDataError {}

#[derive(Debug, Clone)]
/// Parsing Error - PDU provided malformed data
pub struct InvalidDataError;

impl std::fmt::Display for InvalidDataError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "could not find required data")
    }
}

impl std::error::Error for InvalidDataError {}

#[derive(Debug)]
/// A collection of all possible errors
pub enum MPXError {
    Reqwest(reqwest::Error),
    HTMLParser(html_parser::Error),
    ParseIntError(std::num::ParseIntError),
    ParseFloatError(std::num::ParseFloatError),
    EnumParseError(EnumParseError),
    MissingDataError(MissingDataError),
    InvalidDataError(InvalidDataError),
}

impl From<reqwest::Error> for MPXError {
    fn from(e: reqwest::Error) -> Self {
        MPXError::Reqwest(e)
    }
}

impl From<html_parser::Error> for MPXError {
    fn from(e: html_parser::Error) -> Self {
        MPXError::HTMLParser(e)
    }
}

impl From<std::num::ParseIntError> for MPXError {
    fn from(e: std::num::ParseIntError) -> Self {
        MPXError::ParseIntError(e)
    }
}

impl From<std::num::ParseFloatError> for MPXError {
    fn from(e: std::num::ParseFloatError) -> Self {
        MPXError::ParseFloatError(e)
    }
}

impl From<EnumParseError> for MPXError {
    fn from(e: EnumParseError) -> Self {
        MPXError::EnumParseError(e)
    }
}

impl From<MissingDataError> for MPXError {
    fn from(e: MissingDataError) -> Self {
        MPXError::MissingDataError(e)
    }
}

impl From<InvalidDataError> for MPXError {
    fn from(e: InvalidDataError) -> Self {
        MPXError::InvalidDataError(e)
    }
}

#[derive(Copy,Clone,Debug)]
/// Command that can be send to receptacle
pub enum ReceptacleCmd {
    Disable,
    Enable,
    Reboot,
    Identify,
    ResetEnergy,
}

#[derive(Copy,Clone,Debug)]
/// Command that can be send to main module
pub enum PDUCmd {
    TestEvent,
    ResetEnergy,
}

#[derive(Copy,Clone,Debug)]
/// Command that can be send to branch module
pub enum BranchCmd {
    ResetEnergy,
}

#[derive(Copy,Clone,Debug,PartialEq,Serialize)]
/// Wiring Type (1-Phase or 3-Phase)
pub enum WiringType {
    /// 1-Phase / 3 Wire (L, N, PE)
    OnePhase,
    /// 3-Phase / 5 Wire (L1, L2, L3, N, PE)
    ThreePhase,
}

impl FromStr for WiringType {
    type Err = ();

    fn from_str(input: &str) -> Result<WiringType, Self::Err> {
        match input {
            "1-Phase / 3-Wire (L, N, PE)" => Ok(WiringType::OnePhase),
            "3-Phase / 5-Wire (L1, L2, L3, N, PE)" => Ok(WiringType::ThreePhase),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for WiringType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            WiringType::OnePhase => write!(f, "1-Phase"),
            WiringType::ThreePhase => write!(f, "3-Phase"),
        }
    }
}

/// Firmware Version
#[derive(Copy,Clone,Debug,PartialEq,Serialize)]
pub struct FWVersion {
    pub p0: u8,
    pub p1: u8,
    pub p2: u8,
    pub p3: u8,
}

impl FromStr for FWVersion {
    type Err = MPXError;

    fn from_str(input: &str) -> Result<FWVersion, Self::Err> {
        let parts: Vec<&str> = input.split("-").collect();
        if parts.len() == 4 {
            let p0 = parts.get(0).unwrap().parse::<u8>()?;
            let p1 = parts.get(1).unwrap().parse::<u8>()?;
            let p2 = parts.get(2).unwrap().parse::<u8>()?;
            let p3 = parts.get(3).unwrap().parse::<u8>()?;
            Ok(FWVersion { p0: p0, p1: p1, p2: p2, p3: p3 })
        } else {
            Err(MPXError::MissingDataError(MissingDataError))
        }
    }
}

impl std::fmt::Display for FWVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}.{}.{}.{}", self.p0, self.p1, self.p2, self.p3)
    }
}

#[derive(Copy,Clone,Debug,PartialEq,Serialize)]
/// Receptacle type
pub enum ReceptacleType {
    /// Receptacle for C13 connector
    C13,
    /// Receptacle for C19 connector
    C19,
    /// Receptacle for Schuko connector
    Schuko,
}

impl FromStr for ReceptacleType {
    type Err = ();

    fn from_str(input: &str) -> Result<ReceptacleType, Self::Err> {
        match input {
            "IEC 60320 Sheet F C13" => Ok(ReceptacleType::C13),
            "C19" => Ok(ReceptacleType::C19), /* TODO */
            "Schuko" => Ok(ReceptacleType::Schuko), /* TODO */
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for ReceptacleType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ReceptacleType::C13 => write!(f, "C13"),
            ReceptacleType::C19 => write!(f, "C19"),
            ReceptacleType::Schuko => write!(f, "Schuko"),
        }
    }
}

#[derive(Copy,Clone,Debug,PartialEq,Serialize)]
/// Liebert MPX PEM model
pub enum PEMModel {
    /// 1 phase 32A elementary
    EHAEXQ30,
    /// 1 phase 32A monitored
    EHAXXQ30,
    /// 3 phase 16A elementary
    EHAEXT30,
    /// 3 phase 16A monitored
    EHAXXT30,
    /// 3 phase 32A elementary
    EHAEXR30,
    /// 3 phase 32A monitored
    EHAXXR30,
    /// 3 phase 63A elementary
    EHBEXZ30,
    /// 3 phase 63A monitored
    EHBXXZ30,
}

impl FromStr for PEMModel {
    type Err = ();

    fn from_str(input: &str) -> Result<PEMModel, Self::Err> {
        match input {
            "MPXPEM-EHAEXQ30" => Ok(PEMModel::EHAEXQ30),
            "MPXPEM-EHAXXQ30" => Ok(PEMModel::EHAXXQ30),
            "MPXPEM-EHAEXT30" => Ok(PEMModel::EHAEXT30),
            "MPXPEM-EHAXXT30" => Ok(PEMModel::EHAXXT30),
            "MPXPEM-EHAEXR30" => Ok(PEMModel::EHAEXR30),
            "MPXPEM-EHAXXR30" => Ok(PEMModel::EHAXXR30),
            "MPXPEM-EHBEXZ30" => Ok(PEMModel::EHBEXZ30),
            "MPXPEM-EHBXXZ30" => Ok(PEMModel::EHBXXZ30),
            _ => Err(()),
        }
    }
}

#[derive(Copy,Clone,Debug,PartialEq,Serialize)]
/// Liebert MPX BRM model
pub enum BRMModel {
    /// C13 L1 elementary
    EEBC7N1N,
    /// C13 L2 elementary
    EEBC7N2N,
    /// C13 L3 elementary
    EEBC7N3N,
    /// C19 L1 elementary
    EEBC4O1N,
    /// C19 L2 elementary
    EEBC4O2N,
    /// C19 L3 elementary
    EEBC4O3N,
    /// Schuko L1 elementary
    EEBC3P1N,
    /// Schuko L2 elementary
    EEBC3P2N,
    /// Schuko L3 elementary
    EEBC3P3N,
    /// C13 L1 branch-monitored
    EBBC6N1N,
    /// C13 L2 branch-monitored
    EBBC6N2N,
    /// C13 L3 branch-monitored
    EBBC6N3N,
    /// C19 L1 branch-monitored
    EBBC4O1N,
    /// C19 L2 branch-monitored
    EBBC4O2N,
    /// C19 L3 branch-monitored
    EBBC4O3N,
    /// Schuko L1 branch-monitored
    EBBC3P1N,
    /// Schuko L2 branch-monitored
    EBBC3P2N,
    /// Schuko L3 branch-monitored
    EBBC3P3N,
    /// C13 L1 receptacle-managed
    ERBC6N1N,
    /// C13 L2 receptacle-managed
    ERBC6N2N,
    /// C13 L3 receptacle-managed
    ERBC6N3N,
    /// C19 L1 receptacle-managed
    ERBC4O1N,
    /// C19 L2 receptacle-managed
    ERBC4O2N,
    /// C19 L3 receptacle-managed
    ERBC4O3N,
    /// Schuko L1 receptacle-managed
    ERBC3P1N,
    /// Schuko L2 receptacle-managed
    ERBC3P2N,
    /// Schuko L3 receptacle-managed
    ERBC3P3N,
}

impl FromStr for BRMModel {
    type Err = ();

    fn from_str(input: &str) -> Result<BRMModel, Self::Err> {
        match input {
            "MPXBRM-EEBC7N1N" => Ok(BRMModel::EEBC7N1N),
            "MPXBRM-EEBC7N2N" => Ok(BRMModel::EEBC7N2N),
            "MPXBRM-EEBC7N3N" => Ok(BRMModel::EEBC7N3N),
            "MPXBRM-EEBC4O1N" => Ok(BRMModel::EEBC4O1N),
            "MPXBRM-EEBC4O2N" => Ok(BRMModel::EEBC4O2N),
            "MPXBRM-EEBC4O3N" => Ok(BRMModel::EEBC4O3N),
            "MPXBRM-EEBC3P1N" => Ok(BRMModel::EEBC3P1N),
            "MPXBRM-EEBC3P2N" => Ok(BRMModel::EEBC3P2N),
            "MPXBRM-EEBC3P3N" => Ok(BRMModel::EEBC3P3N),
            "MPXBRM-EBBC6N1N" => Ok(BRMModel::EBBC6N1N),
            "MPXBRM-EBBC6N2N" => Ok(BRMModel::EBBC6N2N),
            "MPXBRM-EBBC6N3N" => Ok(BRMModel::EBBC6N3N),
            "MPXBRM-EBBC4O1N" => Ok(BRMModel::EBBC4O1N),
            "MPXBRM-EBBC4O2N" => Ok(BRMModel::EBBC4O2N),
            "MPXBRM-EBBC4O3N" => Ok(BRMModel::EBBC4O3N),
            "MPXBRM-EBBC3P1N" => Ok(BRMModel::EBBC3P1N),
            "MPXBRM-EBBC3P2N" => Ok(BRMModel::EBBC3P2N),
            "MPXBRM-EBBC3P3N" => Ok(BRMModel::EBBC3P3N),
            "MPXBRM-ERBC6N1N" => Ok(BRMModel::ERBC6N1N),
            "MPXBRM-ERBC6N2N" => Ok(BRMModel::ERBC6N2N),
            "MPXBRM-ERBC6N3N" => Ok(BRMModel::ERBC6N3N),
            "MPXBRM-ERBC4O1N" => Ok(BRMModel::ERBC4O1N),
            "MPXBRM-ERBC4O2N" => Ok(BRMModel::ERBC4O2N),
            "MPXBRM-ERBC4O3N" => Ok(BRMModel::ERBC4O3N),
            "MPXBRM-ERBC3P1N" => Ok(BRMModel::ERBC3P1N),
            "MPXBRM-ERBC3P2N" => Ok(BRMModel::ERBC3P2N),
            "MPXBRM-ERBC3P3N" => Ok(BRMModel::ERBC3P3N),
            _ => Err(()),
        }
    }
}

#[derive(Copy,Clone,Debug,PartialEq,Serialize)]
/// Event Type
pub enum EventType {
    ReceptacleOverCurrent,
    ReceptacleLowCurrent,
    BranchLowVoltage,
    BranchOverCurrent,
    BranchLowCurrent,
    BranchFailure,
    BranchBreakerOpen,
    PDULowVoltageL1,
    PDULowVoltageL2,
    PDULowVoltageL3,
    PDUOverCurrentL1,
    PDUOverCurrentL2,
    PDUOverCurrentL3,
    PDULowCurrentL1,
    PDULowCurrentL2,
    PDULowCurrentL3,
    PDUFailure,
    PDUCommunicationFail,
    PDUOverCurrentN,
}

impl FromStr for EventType {
    type Err = ();

    fn from_str(input: &str) -> Result<EventType, Self::Err> {
        match input {
            "Receptacle Over Current" => Ok(EventType::ReceptacleOverCurrent),
            "Receptacle Low Current" => Ok(EventType::ReceptacleLowCurrent),
            "Branch Low Voltage (LN)" => Ok(EventType::BranchLowVoltage),
            "Branch Over Current" => Ok(EventType::BranchOverCurrent),
            "Branch Low Current" => Ok(EventType::BranchLowCurrent),
            "Branch Failure" => Ok(EventType::BranchFailure),
            "Branch Breaker Open" => Ok(EventType::BranchBreakerOpen),
            "PDU Low Voltage L1-N" => Ok(EventType::PDULowVoltageL1),
            "PDU Low Voltage L2-N" => Ok(EventType::PDULowVoltageL2),
            "PDU Low Voltage L3-N" => Ok(EventType::PDULowVoltageL3),
            "PDU Over Current L1" => Ok(EventType::PDUOverCurrentL1),
            "PDU Over Current L2" => Ok(EventType::PDUOverCurrentL2),
            "PDU Over Current L3" => Ok(EventType::PDUOverCurrentL3),
            "PDU Low Current L1" => Ok(EventType::PDULowCurrentL1),
            "PDU Low Current L2" => Ok(EventType::PDULowCurrentL2),
            "PDU Low Current L3" => Ok(EventType::PDULowCurrentL3),
            "PDU Failure" => Ok(EventType::PDUFailure),
            "PDU Communication Fail" => Ok(EventType::PDUCommunicationFail),
            "PDU Neutral Over Current" => Ok(EventType::PDUOverCurrentN),
            _ => Err(()),
        }
    }
}

#[derive(Copy,Clone,Debug,PartialEq,Serialize)]
/// Event Level (e.g. warning or alarm)
pub enum EventLevel {
    OK,
    INFO,
    WARNING,
    ALARM,
}

impl FromStr for EventLevel {
    type Err = ();

    fn from_str(input: &str) -> Result<EventLevel, Self::Err> {
        match input {
            "../../../images/accept.png" => Ok(EventLevel::OK),
            "../../../images/warn.png" => Ok(EventLevel::WARNING),
            "../../../images/information.png" => Ok(EventLevel::INFO),
            "../../../images/err.png" => Ok(EventLevel::ALARM),
            _ => Err(()),
        }
    }
}

#[derive(Debug,PartialEq,Serialize)]
/// PDU Event (e.g. a warning or an alarm)
pub struct Event {
    pub level: EventLevel,
    pub pdu: u8,
    pub branch: u8,
    pub receptacle: u8,
    pub event: EventType,
}

#[derive(Copy,Clone,Debug,PartialEq,Serialize)]
/// Line Source (e.g. L1-N)
pub enum LineSource {
    /// Line Source is L1-N
    L1toN,
    /// Line Source is L2-N
    L2toN,
    /// Line Source is L3-N
    L3toN,
}

impl FromStr for LineSource {
    type Err = ();

    fn from_str(input: &str) -> Result<LineSource, Self::Err> {
        match input {
            "Type L1-N" => Ok(LineSource::L1toN),
            "Type L2-N" => Ok(LineSource::L2toN),
            "Type L3-N" => Ok(LineSource::L3toN),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for LineSource {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LineSource::L1toN => write!(f, "L1-N"),
            LineSource::L2toN => write!(f, "L2-N"),
            LineSource::L3toN => write!(f, "L3-N"),
        }
    }
}


#[derive(Copy,Clone,Debug,PartialEq,Serialize)]
/// Hardware capabilities (measurement / control)
pub enum Capability {
    /// Receptacles can be measured and controlled
    MeasureAndControl,
}

impl FromStr for Capability {
    type Err = ();

    fn from_str(input: &str) -> Result<Capability, Self::Err> {
        match input {
            "All Measurements/Control" => Ok(Capability::MeasureAndControl),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for Capability {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Capability::MeasureAndControl => write!(f, "Measure & Control"),
        }
    }
}

#[derive(Clone,Debug)]
/// Condensed Receptacle Information
pub struct ReceptacleListEntry {
    /// PDU number (usually 1)
    pub pdu: u8,
    /// Branch number (usually 1-6)
    pub branch: u8,
    /// Receptacle number (usually 1-6)
    pub receptacle: u8,
    /// Receptacle state (on or off)
    pub enabled: bool,
    /// Receptacle lock state (locked or unlocked)
    pub locked: bool,
    /// Receptacle health status
    pub status: EventLevel,
    /// Receptacle user label
    pub label: String,
}

#[derive(Clone,Debug)]
/// Internal data structure for a table value with unit
struct TableValue {
    /// value (e.g. "23.42", "0.0")
    value: String,
    /// unit (e.g. "kWH", "VAC" or "sec")
    unit: String,
}

impl TableValue {
    fn get_f32(&self, unit: &str) -> Result<f32,MPXError> {
        if self.unit != unit {
            return Err(MPXError::InvalidDataError(InvalidDataError))
        }

        Ok(self.value.parse::<f32>()?)
    }

    fn get_u32(&self, unit: &str) -> Result<u32,MPXError> {
        if self.unit != unit {
            return Err(MPXError::InvalidDataError(InvalidDataError))
        }

        Ok(self.value.parse::<u32>()?)
    }
}

#[derive(Clone,Debug)]
/// Internal data structure with key-value hashmaps
struct InfoTables {
    status: RawDataTable,
    events: RawDataTable,
    settings: RawDataTable,
    hardware: RawDataTable,
}

#[derive(Clone,Debug,PartialEq,Serialize)]
/// Status from a pem module
pub struct PDUStatus {
    /// accumulated energy in kWh
    pub accumulated_energy: f32,
    /// input power in W
    pub input_power: f32,
    /// voltage L1-N in V AC
    pub voltage_l1_n: f32,
    /// voltage L2-N in V AC
    pub voltage_l2_n: f32,
    /// voltage L3-N in V AC
    pub voltage_l3_n: f32,
    /// current L1 in A AC
    pub current_l1: f32,
    /// current L2 in A AC
    pub current_l2: f32,
    /// current L3 in A AC
    pub current_l3: f32,
    /// current N in A AC
    pub current_n: f32,
    /// current available before alarm L1 in A AC
    pub current_available_to_alarm_l1: f32,
    /// current available before alarm L2 in A AC
    pub current_available_to_alarm_l2: f32,
    /// current available before alarm L3 in A AC
    pub current_available_to_alarm_l3: f32,
    /// line utilization L1 in %
    pub current_utilization_l1: f32,
    /// line utilization L2 in %
    pub current_utilization_l2: f32,
    /// line utilization L3 in %
    pub current_utilization_l3: f32,
    /// line frequency in Hz
    pub line_frequency: f32,
}

impl PDUStatus {
    fn from_table(table: RawDataTable) -> Result<Self,MPXError> {
        Ok(PDUStatus {
            accumulated_energy: table.get("PDU Accumulated Energy").ok_or(MissingDataError)?.get_f32("kWH")?,
            input_power: table.get("PDU Total Input Power").ok_or(MissingDataError)?.get_f32("W")?,
            voltage_l1_n: table.get("PDU Voltage L1-N").ok_or(MissingDataError)?.get_f32("VAC")?,
            voltage_l2_n: table.get("PDU Voltage L2-N").ok_or(MissingDataError)?.get_f32("VAC")?,
            voltage_l3_n: table.get("PDU Voltage L3-N").ok_or(MissingDataError)?.get_f32("VAC")?,
            current_l1: table.get("PDU Current L1").ok_or(MissingDataError)?.get_f32("A AC")?,
            current_l2: table.get("PDU Current L2").ok_or(MissingDataError)?.get_f32("A AC")?,
            current_l3: table.get("PDU Current L3").ok_or(MissingDataError)?.get_f32("A AC")?,
            current_n: table.get("PDU Neutral Current Measurement").ok_or(MissingDataError)?.get_f32("A AC")?,
            current_available_to_alarm_l1: table.get("PDU Available L1 Current Until Alarm").ok_or(MissingDataError)?.get_f32("A AC")?,
            current_available_to_alarm_l2: table.get("PDU Available L2 Current Until Alarm").ok_or(MissingDataError)?.get_f32("A AC")?,
            current_available_to_alarm_l3: table.get("PDU Available L3 Current Until Alarm").ok_or(MissingDataError)?.get_f32("A AC")?,
            current_utilization_l1: table.get("PDU Percent L1 Current Utilization").ok_or(MissingDataError)?.get_f32("%")?,
            current_utilization_l2: table.get("PDU Percent L2 Current Utilization").ok_or(MissingDataError)?.get_f32("%")?,
            current_utilization_l3: table.get("PDU Percent L3 Current Utilization").ok_or(MissingDataError)?.get_f32("%")?,
            line_frequency: table.get("PEM Line Frequency").ok_or(MissingDataError)?.get_f32("Hz")?,
        })
    }
}

#[derive(Clone,Debug,PartialEq,Serialize)]
/// Settings from a pem module
pub struct PDUSettings {
    /// PDU user label
    pub label: String,
    /// PDU asset tag 1
    pub asset_tag_1: String,
    /// PDU asset tag 2
    pub asset_tag_2: String,
    /// N over current alarm threshold in %
    pub n_over_current_alarm_threshold: u32,
    /// N over current warning threshold in %
    pub n_over_current_warning_threshold: u32,
    /// L1 low current alarm threshold in %
    pub l1_low_current_alarm_threshold: u32,
    /// L1 over current alarm threshold in %
    pub l1_over_current_alarm_threshold: u32,
    /// L1 over current warning threshold in %
    pub l1_over_current_warning_threshold: u32,
    /// L2 low current alarm threshold in %
    pub l2_low_current_alarm_threshold: u32,
    /// L2 over current alarm threshold in %
    pub l2_over_current_alarm_threshold: u32,
    /// L2 over current warning threshold in %
    pub l2_over_current_warning_threshold: u32,
    /// L3 low current alarm threshold in %
    pub l3_low_current_alarm_threshold: u32,
    /// L3 over current alarm threshold in %
    pub l3_over_current_alarm_threshold: u32,
    /// L3 over current warning threshold in %
    pub l3_over_current_warning_threshold: u32,
}

impl PDUSettings {
    fn from_table(table: RawDataTable) -> Result<Self,MPXError> {
        Ok(PDUSettings {
            label: table.get("PDU User Assigned Label").ok_or(MissingDataError)?.value.clone(),
            asset_tag_1: table.get("PDU Asset Tag 01").ok_or(MissingDataError)?.value.clone().replace("&nbsp;", ""),
            asset_tag_2: table.get("PDU Asset Tag 02").ok_or(MissingDataError)?.value.clone().replace("&nbsp;", ""),
            n_over_current_alarm_threshold: table.get("Neutral Over Current Alarm Threshold").ok_or(MissingDataError)?.get_u32("%")?,
            n_over_current_warning_threshold: table.get("Neutral Over Current Warning Threshold").ok_or(MissingDataError)?.get_u32("%")?,
            l1_over_current_warning_threshold: table.get("Over Current Warn Threshold L1").ok_or(MissingDataError)?.get_u32("%")?,
            l2_over_current_warning_threshold: table.get("Over Current Warn Threshold L2").ok_or(MissingDataError)?.get_u32("%")?,
            l3_over_current_warning_threshold: table.get("Over Current Warn Threshold L3").ok_or(MissingDataError)?.get_u32("%")?,
            l1_over_current_alarm_threshold: table.get("Over Current Alarm Threshold L1").ok_or(MissingDataError)?.get_u32("%")?,
            l2_over_current_alarm_threshold: table.get("Over Current Alarm Threshold L2").ok_or(MissingDataError)?.get_u32("%")?,
            l3_over_current_alarm_threshold: table.get("Over Current Alarm Threshold L3").ok_or(MissingDataError)?.get_u32("%")?,
            l1_low_current_alarm_threshold: table.get("Low Current Alarm Threshold L1").ok_or(MissingDataError)?.get_u32("%")?,
            l2_low_current_alarm_threshold: table.get("Low Current Alarm Threshold L2").ok_or(MissingDataError)?.get_u32("%")?,
            l3_low_current_alarm_threshold: table.get("Low Current Alarm Threshold L3").ok_or(MissingDataError)?.get_u32("%")?,
        })
    }
}

#[derive(Clone,Debug,PartialEq,Serialize)]
/// Hardware information from a pem module
pub struct PDUHardware {
    /// PEM model description
    pub pem_model: PEMModel,
    /// PEM firmware version
    pub fw_version: FWVersion,
    /// PEM serial number
    pub serial_number: String,
    /// PEM wiring type
    pub wiring_type: WiringType,
    /// PEM rated input voltage in V AC
    pub rated_input_voltage: u32,
    /// PEM rated input current in A AC
    pub rated_input_current: u32,
    /// PEM rated input line frequency in Hz
    pub rated_input_line_frequency: u32,
}

impl PDUHardware {
    fn from_table(table: RawDataTable) -> Result<Self,MPXError> {
        Ok(PDUHardware {
            pem_model: PEMModel::from_str(&table.get("PEM Model").ok_or(MissingDataError)?.value)?,
            wiring_type: WiringType::from_str(&table.get("The PDU input wiring type").ok_or(MissingDataError)?.value)?,
            rated_input_voltage: table.get("Rated Input Line Voltage").ok_or(MissingDataError)?.get_u32("VAC")?,
            rated_input_current: table.get("Rated Input Line Current").ok_or(MissingDataError)?.get_u32("A AC")?,
            rated_input_line_frequency: table.get("Rated Input Line Frequency").ok_or(MissingDataError)?.get_u32("Hz")?,
            fw_version: FWVersion::from_str(&table.get("Firmware Version").ok_or(MissingDataError)?.value)?,
            serial_number: table.get("PEM Serial Number").ok_or(MissingDataError)?.value.clone(),
        })
    }
}

#[derive(Clone,Debug,PartialEq,Serialize)]
/// Event information from a pem module
pub struct PDUEvents {
    pub low_voltage_l1: EventLevel,
    pub low_voltage_l2: EventLevel,
    pub low_voltage_l3: EventLevel,
    pub over_current_l1: EventLevel,
    pub over_current_l2: EventLevel,
    pub over_current_l3: EventLevel,
    pub low_current_l1: EventLevel,
    pub low_current_l2: EventLevel,
    pub low_current_l3: EventLevel,
    pub failure: EventLevel,
    pub communication_fail: EventLevel,
    pub over_current_n: EventLevel,
}

impl PDUEvents {
    fn from_table(table: RawDataTable) -> Result<Self,MPXError> {
        Ok(PDUEvents {
            low_voltage_l1: EventLevel::from_str(&table.get("PDU Low Voltage L1-N").ok_or(MissingDataError)?.value)?,
            low_voltage_l2: EventLevel::from_str(&table.get("PDU Low Voltage L2-N").ok_or(MissingDataError)?.value)?,
            low_voltage_l3: EventLevel::from_str(&table.get("PDU Low Voltage L3-N").ok_or(MissingDataError)?.value)?,
            over_current_l1: EventLevel::from_str(&table.get("PDU Over Current L1").ok_or(MissingDataError)?.value)?,
            over_current_l2: EventLevel::from_str(&table.get("PDU Over Current L2").ok_or(MissingDataError)?.value)?,
            over_current_l3: EventLevel::from_str(&table.get("PDU Over Current L3").ok_or(MissingDataError)?.value)?,
            low_current_l1: EventLevel::from_str(&table.get("PDU Low Current L1").ok_or(MissingDataError)?.value)?,
            low_current_l2: EventLevel::from_str(&table.get("PDU Low Current L2").ok_or(MissingDataError)?.value)?,
            low_current_l3: EventLevel::from_str(&table.get("PDU Low Current L3").ok_or(MissingDataError)?.value)?,
            failure: EventLevel::from_str(&table.get("PDU Failure").ok_or(MissingDataError)?.value)?,
            communication_fail: EventLevel::from_str(&table.get("PDU Communication Fail").ok_or(MissingDataError)?.value)?,
            over_current_n: EventLevel::from_str(&table.get("PDU Neutral Over Current").ok_or(MissingDataError)?.value)?,
        })
    }
}


#[derive(Clone,Debug,PartialEq,Serialize)]
/// Information about a PDU input module
pub struct PDUInfo {
    pub status: PDUStatus,
    pub events: PDUEvents,
    pub settings: PDUSettings,
    pub hardware: PDUHardware,
}

impl PDUInfo {
    fn from_tables(tables: InfoTables) -> Result<Self,MPXError> {
        Ok(PDUInfo {
            status: PDUStatus::from_table(tables.status)?,
            events: PDUEvents::from_table(tables.events)?,
            settings: PDUSettings::from_table(tables.settings)?,
            hardware: PDUHardware::from_table(tables.hardware)?,
        })
    }
}

#[derive(Clone,Debug,PartialEq,Serialize)]
/// Status from a branch module
pub struct BranchStatus {
    /// accumulated energy in kWh
    pub accumulated_energy: f32,
    /// voltage in V AC
    pub voltage: f32,
    /// current in A AC
    pub current: f32,
    /// current available before alarm in A AC
    pub current_available_to_alarm: f32,
    /// line utilization in %
    pub current_utilization: f32,
    /// input power in W
    pub power: f32,
    /// apparent power in VA
    pub apparent_power: f32,
    /// power factor (0-1)
    pub power_factor: f32,
}

impl BranchStatus {
    fn from_table(table: RawDataTable) -> Result<Self,MPXError> {
        Ok(BranchStatus {
            accumulated_energy: table.get("Branch Accumulated Energy").ok_or(MissingDataError)?.get_f32("kWH")?,
            voltage: table.get("Branch Voltage").ok_or(MissingDataError)?.get_f32("VAC")?,
            current: table.get("Branch Current").ok_or(MissingDataError)?.get_f32("A AC")?,
            current_available_to_alarm: table.get("Branch Available Current Until Alarm").ok_or(MissingDataError)?.get_f32("A AC")?,
            current_utilization: table.get("Branch Percent Current Utilization").ok_or(MissingDataError)?.get_f32("%")?,
            power: table.get("Branch Power").ok_or(MissingDataError)?.get_f32("W")?,
            apparent_power: table.get("Branch Apparent Power").ok_or(MissingDataError)?.get_f32("VA")?,
            power_factor: table.get("Branch Power Factor").ok_or(MissingDataError)?.get_f32("&nbsp;")?,
        })
    }
}

#[derive(Clone,Debug,PartialEq,Serialize)]
/// Settings from a branch module
pub struct BranchSettings {
    /// Branch module user label
    pub label: String,
    /// Branch module asset tag 1
    pub asset_tag_1: String,
    /// Branch module asset tag 2
    pub asset_tag_2: String,
    /// over current alarm threshold in %
    pub over_current_alarm_threshold: u32,
    /// over current warning threshold in %
    pub over_current_warning_threshold: u32,
    /// low current alarm threshold in %
    pub low_current_alarm_threshold: u32,
}

impl BranchSettings {
    fn from_table(table: RawDataTable) -> Result<Self,MPXError> {
        Ok(BranchSettings {
            label: table.get("Branch User Assigned Label").ok_or(MissingDataError)?.value.clone(),
            asset_tag_1: table.get("Branch Asset Tag 01").ok_or(MissingDataError)?.value.clone().replace("&nbsp;", ""),
            asset_tag_2: table.get("Branch Asset Tag 02").ok_or(MissingDataError)?.value.clone().replace("&nbsp;", ""),
            over_current_alarm_threshold: table.get("Over Current Alarm Threshold").ok_or(MissingDataError)?.get_u32("%")?,
            over_current_warning_threshold: table.get("Over Current Warning Threshold").ok_or(MissingDataError)?.get_u32("%")?,
            low_current_alarm_threshold: table.get("Low Current Alarm Threshold").ok_or(MissingDataError)?.get_u32("%")?,
        })
    }
}

#[derive(Clone,Debug,PartialEq,Serialize)]
/// Hardware information from a branch module
pub struct BranchHardware {
    /// BRM model description
    pub brm_model: BRMModel,
    /// BRM firmware version
    pub fw_version: FWVersion,
    /// BRM serial number
    pub serial_number: String,
    /// Branch module receptacle type
    pub receptacle_type: ReceptacleType,
    /// Branch module capabilities
    pub capabilities: Capability,
    /// Line source
    pub line_source: LineSource,
    /// Rated line voltage in V AC
    pub rated_line_voltage: u32,
    /// Rated line current in A AC
    pub rated_line_current: u32,
    /// Rated line current in Hz
    pub rated_line_frequency: u32,
}

impl BranchHardware {
    fn from_table(table: RawDataTable) -> Result<Self,MPXError> {
        Ok(BranchHardware {
            brm_model: BRMModel::from_str(&table.get("BRM Model").ok_or(MissingDataError)?.value)?,
            receptacle_type: ReceptacleType::from_str(&table.get("Branch Receptacle Type").ok_or(MissingDataError)?.value)?,
            capabilities: Capability::from_str(&table.get("Branch Capabilities").ok_or(MissingDataError)?.value)?,
            line_source: LineSource::from_str(&table.get("Branch Line Source").ok_or(MissingDataError)?.value)?,
            rated_line_voltage: table.get("Branch Rated Line Voltage").ok_or(MissingDataError)?.get_u32("VAC")?,
            rated_line_current: table.get("Branch Rated Line Current").ok_or(MissingDataError)?.get_u32("A AC")?,
            rated_line_frequency: table.get("Branch Rated Line Frequency").ok_or(MissingDataError)?.get_u32("Hz")?,
            fw_version: FWVersion::from_str(&table.get("Firmware Version").ok_or(MissingDataError)?.value)?,
            serial_number: table.get("Branch Serial Number").ok_or(MissingDataError)?.value.clone(),
        })
    }
}

#[derive(Clone,Debug,PartialEq,Serialize)]
/// Event information from a branch module
pub struct BranchEvents {
    pub low_voltage: EventLevel,
    pub over_current: EventLevel,
    pub low_current: EventLevel,
    pub failure: EventLevel,
    pub breaker_open: EventLevel,
}

impl BranchEvents {
    fn from_table(table: RawDataTable) -> Result<Self,MPXError> {
        Ok(BranchEvents {
            low_voltage: EventLevel::from_str(&table.get("Branch Low Voltage (LN)").ok_or(MissingDataError)?.value)?,
            over_current: EventLevel::from_str(&table.get("Branch Over Current").ok_or(MissingDataError)?.value)?,
            low_current: EventLevel::from_str(&table.get("Branch Low Current").ok_or(MissingDataError)?.value)?,
            failure: EventLevel::from_str(&table.get("Branch Failure").ok_or(MissingDataError)?.value)?,
            breaker_open: EventLevel::from_str(&table.get("Branch Breaker Open").ok_or(MissingDataError)?.value)?,
        })
    }
}

#[derive(Clone,Debug,PartialEq,Serialize)]
/// Information about a branch module
pub struct BranchInfo {
    pub status: BranchStatus,
    pub events: BranchEvents,
    pub settings: BranchSettings,
    pub hardware: BranchHardware,
}

impl BranchInfo {
    fn from_tables(tables: InfoTables) -> Result<Self,MPXError> {
        Ok(BranchInfo {
            status: BranchStatus::from_table(tables.status)?,
            events: BranchEvents::from_table(tables.events)?,
            settings: BranchSettings::from_table(tables.settings)?,
            hardware: BranchHardware::from_table(tables.hardware)?,
        })
    }
}

#[derive(Clone,Debug,PartialEq,Serialize)]
/// Status from a receptacle
pub struct ReceptacleStatus {
    /// accumulated energy in kWh
    pub accumulated_energy: f32,
    /// voltage in V AC
    pub voltage: f32,
    /// current in A AC
    pub current: f32,
    /// current available before alarm in A AC
    pub current_available_to_alarm: f32,
    /// line utilization in %
    pub current_utilization: f32,
    /// input power in W
    pub power: f32,
    /// apparent power in VA
    pub apparent_power: f32,
    /// power factor (0-1)
    pub power_factor: f32,
    /// current crest factor (0-1)
    pub current_crest_factor: f32,
}

impl ReceptacleStatus {
    fn from_table(table: RawDataTable) -> Result<Self,MPXError> {
        Ok(ReceptacleStatus {
            accumulated_energy: table.get("Receptacle Accumulated Energy").ok_or(MissingDataError)?.get_f32("kWH")?,
            voltage: table.get("Receptacle Voltage").ok_or(MissingDataError)?.get_f32("VAC")?,
            current: table.get("Receptacle Current").ok_or(MissingDataError)?.get_f32("A AC")?,
            current_available_to_alarm: table.get("Receptacle Available Current Until Alarm").ok_or(MissingDataError)?.get_f32("A AC")?,
            current_utilization: table.get("Receptacle Percent Current Utilization").ok_or(MissingDataError)?.get_f32("%")?,
            power: table.get("Receptacle Power").ok_or(MissingDataError)?.get_f32("W")?,
            apparent_power: table.get("Receptacle Apparent Power").ok_or(MissingDataError)?.get_f32("VA")?,
            power_factor: table.get("Receptacle Power Factor").ok_or(MissingDataError)?.get_f32("&nbsp;")?,
            current_crest_factor: table.get("Receptacle Current Crest Factor").ok_or(MissingDataError)?.get_f32("&nbsp;")?,
        })
    }
}

#[derive(Clone,Debug,PartialEq,Serialize)]
/// Settings from a receptacle
pub struct ReceptacleSettings {
    /// Receptacle user label
    pub label: String,
    /// Receptacle module asset tag 1
    pub asset_tag_1: String,
    /// Receptacle module asset tag 2
    pub asset_tag_2: String,
    /// over current alarm threshold in %
    pub over_current_alarm_threshold: u32,
    /// over current warning threshold in %
    pub over_current_warning_threshold: u32,
    /// low current alarm threshold in %
    pub low_current_alarm_threshold: u32,
    /// current power state (true=enabled, false=disabled)
    pub power_state: bool,
    /// requested power state (true=enabled, false=disabled)
    pub power_control: bool,
    /// lock state (true=locked, false=unlocked)
    pub control_lock_state: bool,
    /// power on delay in seconds
    pub power_on_delay: u32,
}

impl ReceptacleSettings {
    fn from_table(table: RawDataTable) -> Result<Self,MPXError> {
        Ok(ReceptacleSettings {
            label: table.get("Receptacle User Assigned Label").ok_or(MissingDataError)?.value.clone(),
            asset_tag_1: table.get("Receptacle Asset Tag 01").ok_or(MissingDataError)?.value.clone().replace("&nbsp;", ""),
            asset_tag_2: table.get("Receptacle Asset Tag 02").ok_or(MissingDataError)?.value.clone().replace("&nbsp;", ""),
            over_current_alarm_threshold: table.get("Over Current Alarm Threshold").ok_or(MissingDataError)?.get_u32("%")?,
            over_current_warning_threshold: table.get("Over Current Warning Threshold").ok_or(MissingDataError)?.get_u32("%")?,
            low_current_alarm_threshold: table.get("Low Current Alarm Threshold").ok_or(MissingDataError)?.get_u32("%")?,
            power_state: table.get("Receptacle Power State").ok_or(MissingDataError)?.value == "On",
            power_control: table.get("Receptacle Power Control").ok_or(MissingDataError)?.value == "On",
            control_lock_state: table.get("Receptacle Control Lock State").ok_or(MissingDataError)?.value == "Locked",
            power_on_delay: table.get("Receptacle Power On Delay").ok_or(MissingDataError)?.get_u32("sec")?,
        })
    }
}

#[derive(Clone,Debug,PartialEq,Serialize)]
/// Hardware information from a receptacle
pub struct ReceptacleHardware {
    /// Receptacle type (e.g. C13 or Schuko)
    pub receptacle_type: ReceptacleType,
    /// Line Source (e.g. L1-N or L2-N)
    pub line_source: LineSource,
    /// Receptacle capabilities (e.g. controllable)
    pub capabilities: Capability,
}

impl ReceptacleHardware {
    fn from_table(table: RawDataTable) -> Result<Self,MPXError> {
        Ok(ReceptacleHardware {
            receptacle_type: ReceptacleType::from_str(&table.get("Receptacle Type").ok_or(MissingDataError)?.value)?,
            line_source: LineSource::from_str(&table.get("Receptacle Line Source").ok_or(MissingDataError)?.value)?,
            capabilities: Capability::from_str(&table.get("Receptacle Capabilities").ok_or(MissingDataError)?.value)?,
        })
    }
}

#[derive(Clone,Debug,PartialEq,Serialize)]
/// Event information from a receptacle
pub struct ReceptacleEvents {
    pub over_current: EventLevel,
    pub low_current: EventLevel,
}

impl ReceptacleEvents {
    fn from_table(table: RawDataTable) -> Result<Self,MPXError> {
        Ok(ReceptacleEvents {
            over_current: EventLevel::from_str(&table.get("Receptacle Over Current").ok_or(MissingDataError)?.value)?,
            low_current: EventLevel::from_str(&table.get("Receptacle Low Current").ok_or(MissingDataError)?.value)?,
        })
    }
}

#[derive(Clone,Debug,PartialEq,Serialize)]
/// Information about a Receptacle
pub struct ReceptacleInfo {
    pub status: ReceptacleStatus,
    pub events: ReceptacleEvents,
    pub settings: ReceptacleSettings,
    pub hardware: ReceptacleHardware,
}

impl ReceptacleInfo {
    fn from_tables(tables: InfoTables) -> Result<Self,MPXError> {
        Ok(ReceptacleInfo {
            status: ReceptacleStatus::from_table(tables.status)?,
            events: ReceptacleEvents::from_table(tables.events)?,
            settings: ReceptacleSettings::from_table(tables.settings)?,
            hardware: ReceptacleHardware::from_table(tables.hardware)?,
        })
    }
}

/// Representation of a Liebert MPX PDU
pub struct MPX {
    host: String,
    username: String,
    password: String,
}

impl MPX {
    pub fn new(host: &str, username: &str, password: &str) -> Self {
        MPX{
            host: host.to_string(),
            username: username.to_string(),
            password: password.to_string(),
        }
    }
}

fn parse_receptacle_list_row(row: &html_parser::Element) -> Result<ReceptacleListEntry, MPXError> {
    let rowid: Vec<&str> = row.id.as_ref().unwrap().split("-").collect();

    if rowid.len() != 3 {
        return Err(MPXError::InvalidDataError(InvalidDataError))
    }

    let pdu = rowid.get(0).unwrap().parse::<u8>()?;
    let branch = rowid.get(1).unwrap().parse::<u8>()?;
    let receptacle = rowid.get(2).unwrap().parse::<u8>()?;

    let label = match row.children.get(0) {
        Some(html_parser::Node::Element(td)) => {
            match td.children.get(0) {
                Some(html_parser::Node::Element(a)) => {
                    match a.children.get(0) {
                        Some(html_parser::Node::Element(nobr)) => {
                            match nobr.children.get(0) {
                                Some(html_parser::Node::Text(text)) => {
                                    text.clone()
                                },
                                _ => {
                                    return Err(MPXError::InvalidDataError(InvalidDataError))
                                },
                            }
                        },
                        _ => {
                            return Err(MPXError::InvalidDataError(InvalidDataError))
                        },
                    }
                },
                _ => {
                    return Err(MPXError::InvalidDataError(InvalidDataError))
                },
            }
        }
        _ => {
            return Err(MPXError::InvalidDataError(InvalidDataError))
        },
    };

    let state = match row.children.get(2) {
        Some(html_parser::Node::Element(td)) => {
            match td.children.get(0) {
                Some(html_parser::Node::Element(span)) => {
                    match span.attributes.get("title").unwrap_or(&None).as_ref().unwrap_or(&"".to_string()).as_str() {
                        "On" => true,
                        "Off" => false,
                        _ => {
                            return Err(MPXError::InvalidDataError(InvalidDataError))
                        },
                    }
                }
                _ => {
                    return Err(MPXError::InvalidDataError(InvalidDataError))
                },
            }
        },
        _ => {
            return Err(MPXError::InvalidDataError(InvalidDataError))
        },
    };

    let locked = match row.children.get(3) {
        Some(html_parser::Node::Element(td)) => {
            match td.children.get(0) {
                Some(html_parser::Node::Element(span)) => {
                    match span.attributes.get("title").unwrap_or(&None).as_ref().unwrap_or(&"".to_string()).as_str() {
                        "Unlocked" => false,
                        "Locked" => true,
                        _ => {
                            return Err(MPXError::InvalidDataError(InvalidDataError))
                        },
                    }
                }
                _ => {
                    return Err(MPXError::InvalidDataError(InvalidDataError))
                },
            }
        },
        _ => {
            return Err(MPXError::InvalidDataError(InvalidDataError))
        },
    };

    let status = match row.children.get(4) {
        Some(html_parser::Node::Element(td)) => {
            match td.children.get(0) {
                Some(html_parser::Node::Element(img)) => {
                    EventLevel::from_str(img.attributes.get("src").unwrap_or(&None).as_ref().unwrap_or(&"".to_string()).as_str())?
                }
                _ => {
                    return Err(MPXError::InvalidDataError(InvalidDataError))
                },
            }
        },
        _ => {
            return Err(MPXError::InvalidDataError(InvalidDataError))
        },
    };

    Ok(ReceptacleListEntry {
        pdu: pdu,
        branch: branch,
        receptacle: receptacle,
        enabled: state,
        locked: locked,
        status: status,
        label: label,
    })
}

fn parse_receptacles(html: String) -> Result<ReceptacleList, MPXError> {
    let dom = html_parser::Dom::parse(&html)?;
    let mut result = Vec::new();

    for child in dom.children.iter() {
        match child {
            html_parser::Node::Element(e) => {
                if e.id == Some("rcpTable".to_string()) && e.name == "table" {
                    for row_raw in e.children.iter() {
                        match row_raw {
                            html_parser::Node::Element(row) => {
                                if row.name == "tr" && row.id.is_some() {
                                    result.push(parse_receptacle_list_row(row)?);
                                }
                            }
                            _ => {
                                return Err(MPXError::InvalidDataError(InvalidDataError));
                            },
                        }
                    }
                }
            },
            _ => {
                return Err(MPXError::InvalidDataError(InvalidDataError));
            },
        }
    }

    Ok(result)
}

impl MPX {
    pub async fn get_receptacles(self: &Self) -> Result<ReceptacleList, MPXError> {
        let url = format!("http://{}/rpc/rpcReceptacleListData.htm", self.host);
        let html = reqwest::get(url).await?.text().await?;
        parse_receptacles(html)
    }
}

fn parse_event_row(row: &html_parser::Element) -> Result<Option<Event>, MPXError> {
    let colnode0 = row.children.get(0).ok_or(InvalidDataError)?;

    let level = match colnode0 {
        html_parser::Node::Element(cell) => {
            if cell.name == "th" {
                return Ok(None);
            }

            match get_child_text(colnode0) {
                Some(text) => {
                    if text == "No Alarms Present" {
                        return Ok(None);
                    }
                }
                _ => {},
            }

            let imgnode = get_child_node(colnode0, "img").ok_or(InvalidDataError)?;

            match imgnode {
                html_parser::Node::Element(img) => {
                    let src = img.attributes.get("src").ok_or(InvalidDataError)?;
                    let src = src.as_ref().ok_or(InvalidDataError)?;

                    EventLevel::from_str(src)
                },
                _ => {
                    return Err(MPXError::InvalidDataError(InvalidDataError));
                },
            }
        },
        _ => {
            return Err(MPXError::InvalidDataError(InvalidDataError));
        },
    }?;

    let colnode1 = row.children.get(1).ok_or(InvalidDataError)?;
    let colnode2 = row.children.get(2).ok_or(InvalidDataError)?;

    let id = get_child_text(colnode1).ok_or(InvalidDataError)?;
    let event = get_child_text(colnode2).ok_or(InvalidDataError)?;

    let defaultid = "0";
    let id: Vec<&str> = id.split("-").collect();
    let pdu = id.get(0).unwrap_or(&defaultid).parse::<u8>()?;
    let branch = id.get(1).unwrap_or(&defaultid).parse::<u8>()?;
    let receptacle = id.get(2).unwrap_or(&defaultid).parse::<u8>()?;


    Ok(Some(Event {
        pdu: pdu,
        branch: branch,
        receptacle: receptacle,
        level: level,
        event: EventType::from_str(event)?,

    }))
}

fn get_child_text<'a>(node: &'a html_parser::Node) -> Option<&'a String> {
    match node {
        html_parser::Node::Element(e) => {
            for child in e.children.iter() {
                match child {
                    html_parser::Node::Text(t) => {
                        return Some(t);
                    },
                    _ => continue,
                }
            };
        },
        _ => return None,
    };

    None
}

fn get_child_node<'a>(node: &'a html_parser::Node, name: &str) -> Option<&'a html_parser::Node> {
    match node {
        html_parser::Node::Element(e) => {
            for child in e.children.iter() {
                match child {
                    html_parser::Node::Element(c) => {
                        if c.name == name {
                            return Some(child);
                        }
                    },
                    _ => continue,
                }
            };
        },
        _ => return None,
    };

    None
}

fn get_child_node_by_id<'a>(node: &'a html_parser::Node, name: &str, id: &str) -> Option<&'a html_parser::Node> {
    match node {
        html_parser::Node::Element(e) => {
            for child in e.children.iter() {
                match child {
                    html_parser::Node::Element(c) => {
                        if c.name == name && c.id == Some(id.to_string()) {
                            return Some(child);
                        }
                    },
                    _ => continue,
                }
            };
        },
        _ => return None,
    };

    None
}

fn parse_table<'a>(node: &'a html_parser::Node, alarm: bool) -> Result<RawDataTable, MPXError> {
    let mut result = HashMap::new();

    match node {
        html_parser::Node::Element(table) => {
            for rownode in table.children.iter() {
                match rownode {
                    html_parser::Node::Element(row) => {
                        if row.name == "tr" {
                            let keynode = row.children.get(if alarm { 1 } else { 0 }).ok_or(InvalidDataError)?;
                            match keynode {
                                html_parser::Node::Element(e) => {
                                    if e.name == "th" {
                                        continue;
                                    }
                                },
                                _ => {},
                            }
                            let key = get_child_text(keynode).ok_or(InvalidDataError)?;

                            let valuenode = row.children.get(if alarm { 0 } else { 1 }).ok_or(InvalidDataError)?;
                            let value = if !alarm {
                                get_child_text(valuenode).ok_or(InvalidDataError)?
                            } else {
                                let valuenode = get_child_node(valuenode, "img").ok_or(InvalidDataError)?;
                                match valuenode {
                                    html_parser::Node::Element(e) => {
                                        let src = e.attributes.get("src").ok_or(InvalidDataError)?;
                                        src.as_ref().ok_or(InvalidDataError)?
                                    },
                                    _ => {
                                        return Err(MPXError::InvalidDataError(InvalidDataError));
                                    },
                                }
                            };

                            let empty = "".to_string();
                            let unitnode = row.children.get(2).ok_or(InvalidDataError)?;
                            let unit = if !alarm {
                                get_child_text(unitnode).ok_or(InvalidDataError)?
                            } else {
                                &empty
                            };

                            result.insert(
                                key.clone(),
                                TableValue { value: value.clone(), unit: unit.clone() }
                            );
                        }
                    },
                    _ => {},
                }
            }

            Ok(result)
        },
        _ => Err(MPXError::InvalidDataError(InvalidDataError))
    }
}

fn get_info_tables(html: String) -> Result<InfoTables, MPXError> {
    let dom = html_parser::Dom::parse(&html)?;

    let html_node = dom.children.get(0).ok_or(InvalidDataError)?;
    let body_node = get_child_node(html_node, "body").ok_or(InvalidDataError)?;

    let status_node = get_child_node_by_id(body_node, "div", "RpcStatusArea").ok_or(InvalidDataError)?;
    let status_node = get_child_node(status_node, "table").ok_or(InvalidDataError)?;

    let alarm_node = get_child_node_by_id(body_node, "div", "RpcAlarmArea").ok_or(InvalidDataError)?;
    let alarm_node = get_child_node(alarm_node, "table").ok_or(InvalidDataError)?;

    let settings_node = get_child_node_by_id(body_node, "div", "RpcSettingArea").ok_or(InvalidDataError)?;
    let settings_node = get_child_node(settings_node, "table").ok_or(InvalidDataError)?;

    let hardware_node = get_child_node_by_id(body_node, "div", "RpcInfoArea").ok_or(InvalidDataError)?;
    let hardware_node = get_child_node(hardware_node, "table").ok_or(InvalidDataError)?;

    Ok(InfoTables {
        status: parse_table(status_node, false)?,
        events: parse_table(alarm_node, true)?,
        settings: parse_table(settings_node, false)?,
        hardware: parse_table(hardware_node, false)?,
    })
}

fn parse_events(html: String)  -> Result<EventList, MPXError> {
    let dom = html_parser::Dom::parse(&html)?;
    let mut result = Vec::new();

    let html_node = dom.children.get(0).ok_or(InvalidDataError)?;
    let body_node = get_child_node(html_node, "body").ok_or(InvalidDataError)?;

    let detail_node = get_child_node_by_id(body_node, "div", "DetailPanelArea").ok_or(InvalidDataError)?;
    let table_node = get_child_node(detail_node, "table").ok_or(InvalidDataError)?;

    match table_node {
        html_parser::Node::Element(table) => {
            for rownode in table.children.iter() {
                match rownode {
                    html_parser::Node::Element(row) => {
                        if row.name == "tr" {
                            let re = parse_event_row(row)?;
                            if re.is_some() {
                                result.push(re.unwrap());
                            }
                        }
                    }
                    _ => {
                        return Err(MPXError::InvalidDataError(InvalidDataError));
                    }
                }
            }
        }
        _ => {
            return Err(MPXError::InvalidDataError(InvalidDataError));
        },
    }

    Ok(result)
}

impl MPX {
    pub async fn get_events(self: &Self) -> Result<EventList, MPXError> {
        let url = format!("http://{}/rpc/rpcActiveAlarms.htm", self.host);
        let html = reqwest::get(url).await?.text().await?;
        parse_events(html)
    }

    pub async fn get_info_pdu(self: &Self, pdu: u8) -> Result<PDUInfo, MPXError> {
        let url = format!("http://{}/dp/std:{}.0.0_0.0.0/rpc/rpcAps.htm", self.host, pdu);
        let html = reqwest::get(url).await?.text().await?;
        PDUInfo::from_tables(get_info_tables(html)?)
    }

    pub async fn get_info_branch(self: &Self, pdu: u8, branch: u8) -> Result<BranchInfo, MPXError> {
        let url = format!("http://{}/dp/std:{}.{}.0_0.0.0/rpc/rpcRem.htm", self.host, pdu, branch);
        let html = reqwest::get(url).await?.text().await?;
        BranchInfo::from_tables(get_info_tables(html)?)
    }

    pub async fn get_info_receptacle(self: &Self, pdu: u8, branch: u8, receptacle: u8) -> Result<ReceptacleInfo, MPXError> {
        let url = format!("http://{}/dp/std:{}.{}.{}_0.0.0/rpc/rpcReceptacle.htm", self.host, pdu, branch, receptacle);
        let html = reqwest::get(url).await?.text().await?;
        ReceptacleInfo::from_tables(get_info_tables(html)?)
    }

    async fn send_query(self: &Self, url: String, params: &[(&str, &str)]) -> Result<(), MPXError> {
        let client = reqwest::Client::new();
        let response = client.post(url)
            .basic_auth(self.username.clone(), Some(self.password.clone()))
            .form(params)
            .send()
            .await?;

        if response.status() != reqwest::StatusCode::OK && response.status() != reqwest::StatusCode::SEE_OTHER {
            return Err(MPXError::InvalidDataError(InvalidDataError))
        }

        Ok(())
    }

    pub async fn pdu_command(self: &Self, pdu: u8, cmd: PDUCmd) -> Result<(), MPXError> {
        let url = format!("http://{}/dp/std:{}.0.0_0.0.0/rpc/rpcControlApsCommand", self.host, pdu);
        match cmd {
            PDUCmd::TestEvent => self.send_query(url, &[("testEvent", "Send")]).await,
            PDUCmd::ResetEnergy => self.send_query(url, &[("energyControl", "Reset")]).await,
        }
    }

    pub async fn pdu_reset_energy(self: &Self, pdu: u8) -> Result<(), MPXError> {
        self.pdu_command(pdu, PDUCmd::ResetEnergy).await
    }

    pub async fn pdu_test_event(self: &Self, pdu: u8) -> Result<(), MPXError> {
        self.pdu_command(pdu, PDUCmd::TestEvent).await
    }

    pub async fn branch_command(self: &Self, pdu: u8, branch: u8, cmd: BranchCmd) -> Result<(), MPXError> {
        let url = format!("http://{}/dp/std:{}.{}.0_0.0.0/rpc/rpcControlRemCommand", self.host, pdu, branch);
        match cmd {
            BranchCmd::ResetEnergy => self.send_query(url, &[("energyControl", "Reset")]).await,
        }
    }

    pub async fn branch_reset_energy(self: &Self, pdu: u8, branch: u8) -> Result<(), MPXError> {
        self.branch_command(pdu, branch, BranchCmd::ResetEnergy).await
    }

    pub async fn receptacle_command(self: &Self, pdu: u8, branch: u8, port: u8, cmd: ReceptacleCmd) -> Result<(), MPXError> {
        let url = format!("http://{}/dp/std:{}.{}.{}_0.0.0/rpc/rpcControlReceptacleCommand", self.host, pdu, branch, port);
        match cmd {
            ReceptacleCmd::Disable => self.send_query(url, &[("receptacleStateGroup", "0"), ("Submit", "Save")]),
            ReceptacleCmd::Enable => self.send_query(url, &[("receptacleStateGroup", "1"), ("Submit", "Save")]),
            ReceptacleCmd::Reboot => self.send_query(url, &[("receptacleStateGroup", "2"), ("Submit", "Save")]),
            ReceptacleCmd::Identify => self.send_query(url, &[("rcpIdentControl", "Submit")]),
            ReceptacleCmd::ResetEnergy => self.send_query(url, &[("energyControl", "Reset")]),
        }.await
    }

    pub async fn receptacle_identify(self: &Self, pdu: u8, branch: u8, port: u8) -> Result<(), MPXError> {
        self.receptacle_command(pdu, branch, port, ReceptacleCmd::Identify).await
    }

    pub async fn receptacle_reboot(self: &Self, pdu: u8, branch: u8, port: u8) -> Result<(), MPXError> {
        self.receptacle_command(pdu, branch, port, ReceptacleCmd::Reboot).await
    }

    pub async fn receptacle_enable(self: &Self, pdu: u8, branch: u8, port: u8) -> Result<(), MPXError> {
        self.receptacle_command(pdu, branch, port, ReceptacleCmd::Enable).await
    }

    pub async fn receptacle_disable(self: &Self, pdu: u8, branch: u8, port: u8) -> Result<(), MPXError> {
        self.receptacle_command(pdu, branch, port, ReceptacleCmd::Disable).await
    }

    pub async fn receptacle_reset_energy(self: &Self, pdu: u8, branch: u8, port: u8) -> Result<(), MPXError> {
        self.receptacle_command(pdu, branch, port, ReceptacleCmd::ResetEnergy).await
    }

    pub async fn set_pdu_settings(self: &Self, pdu: u8, settings: &PDUSettings) -> Result<(), MPXError> {
        let url = format!("http://{}/dp/std:{}.0.0_0.0.0/rpc/rpcControlApsSetting", self.host, pdu);
        let parameters = [
            ("Submit", "Save"),
            ("label", &settings.label),
            ("assetTag1", &settings.asset_tag_1),
            ("assetTag2", &settings.asset_tag_2),
            ("ecNeutralThrshldOverAlarm", &format!("{}", settings.n_over_current_alarm_threshold)),
            ("ecNeutralThrshldOverWarn", &format!("{}", settings.n_over_current_warning_threshold)),
            ("ecThresholdHiAlmL1", &format!("{}", settings.l1_over_current_alarm_threshold)),
            ("ecThresholdHiAlmL2", &format!("{}", settings.l2_over_current_alarm_threshold)),
            ("ecThresholdHiAlmL3", &format!("{}", settings.l3_over_current_alarm_threshold)),
            ("ecThresholdHiWrnL1", &format!("{}", settings.l1_over_current_warning_threshold)),
            ("ecThresholdHiWrnL2", &format!("{}", settings.l2_over_current_warning_threshold)),
            ("ecThresholdHiWrnL3", &format!("{}", settings.l3_over_current_warning_threshold)),
            ("ecThresholdLoAlmL1", &format!("{}", settings.l1_low_current_alarm_threshold)),
            ("ecThresholdLoAlmL2", &format!("{}", settings.l2_low_current_alarm_threshold)),
            ("ecThresholdLoAlmL3", &format!("{}", settings.l3_low_current_alarm_threshold)),
        ];
        self.send_query(url, &parameters).await
    }

    pub async fn set_branch_settings(self: &Self, pdu: u8, branch: u8, settings: &BranchSettings) -> Result<(), MPXError> {
        let url = format!("http://{}/dp/std:{}.{}.0_0.0.0/rpc/rpcControlRemSetting", self.host, pdu, branch);
        let parameters = [
            ("Submit", "Save"),
            ("label", &settings.label),
            ("assetTag1", &settings.asset_tag_1),
            ("assetTag2", &settings.asset_tag_2),
            ("ecThresholdHiAlmLN", &format!("{}", settings.over_current_alarm_threshold)),
            ("ecThresholdHiWrnLN", &format!("{}", settings.over_current_warning_threshold)),
            ("ecThresholdLoAlmLN", &format!("{}", settings.low_current_alarm_threshold)),
        ];
        self.send_query(url, &parameters).await
    }

    pub async fn set_receptacle_settings(self: &Self, pdu: u8, branch: u8, receptacle: u8, settings: &ReceptacleSettings) -> Result<(), MPXError> {
        let url = format!("http://{}/dp/std:{}.{}.{}_0.0.0/rpc/rpcControlReceptacleSetting", self.host, pdu, branch, receptacle);
        let parameters = [
            ("Submit", "Save"),
            ("label", &settings.label),
            ("assetTag1", &settings.asset_tag_1),
            ("assetTag2", &settings.asset_tag_2),
            ("ecThresholdHiAlmL1", &format!("{}", settings.over_current_alarm_threshold)),
            ("ecThresholdHiWrnL1", &format!("{}", settings.over_current_warning_threshold)),
            ("ecThresholdLoAlmL1", &format!("{}", settings.low_current_alarm_threshold)),
            ("powerUpDelay", &format!("{}", settings.power_on_delay)),
            ("lockStateTypeGroup1", if settings.control_lock_state { "1" } else { "0" }),
        ];
        self.send_query(url, &parameters).await
    }
}

#[cfg(test)]
mod parser_unit_tests {
    use super::*;

    #[test]
    fn test_01_parse_receptacles() {
        let html = include_str!("../testdata/receptacle-list.htm").to_string();
        let parsed = parse_receptacles(html);

        assert!(parsed.is_ok())
    }

    #[test]
    fn test_02_parse_events_empty() {
        let html = include_str!("../testdata/events-none.htm").to_string();
        let parsed = parse_events(html);

        assert!(parsed.is_ok());
    }

    #[test]
    fn test_03_parse_events_test() {
        let html = include_str!("../testdata/events-test.htm").to_string();
        let parsed = parse_events(html);

        assert!(parsed.is_ok());
    }

    #[test]
    fn test_04_parse_pdu_info() {
        let html = include_str!("../testdata/pdu-info.htm").to_string();
        let tables = get_info_tables(html);
        assert!(tables.is_ok(), "failed to get info tables");

        if tables.is_ok() {
            let info = PDUInfo::from_tables(tables.unwrap());
            assert!(info.is_ok(), "failed to get PDUInfo");
        }
    }

    #[test]
    fn test_05_parse_branch_info() {
        let html = include_str!("../testdata/branch-info.htm").to_string();
        let tables = get_info_tables(html);
        assert!(tables.is_ok(), "failed to get info tables");

        if tables.is_ok() {
            let info = BranchInfo::from_tables(tables.unwrap());
            assert!(info.is_ok(), "failed to get BranchInfo");
        }
    }

    #[test]
    fn test_06_parse_receptacle_info() {
        let html = include_str!("../testdata/receptacle-info.htm").to_string();
        let tables = get_info_tables(html);
        assert!(tables.is_ok(), "failed to get info tables");

        if tables.is_ok() {
            let info = ReceptacleInfo::from_tables(tables.unwrap());
            assert!(info.is_ok(), "failed to get ReceptacleInfo");
        }
    }
}
