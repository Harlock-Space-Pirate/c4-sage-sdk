use crate::bytes::encode_pk;
use crate::catalog::catalog;

/// Fleet readout from live account offsets. TOTAL HULL is not on-chain.
#[derive(Debug, Clone)]
pub struct FleetReadout {
    pub fleet_label: String,
    pub owner_profile: String,
    pub sub_profile: String,
    pub required_crew: u16,
    pub passenger_capacity: u16,
    pub respawn_time_without_fee: u16,
    pub crew_count: u16,
    pub rented_crew: u16,
    pub ap: u32,
    pub fuel_id: u16,
    pub fuel_amount: u64,
    pub ammo_id: u16,
    pub ammo_amount: u64,
}

fn u16_at(d: &[u8], o: usize) -> u16 {
    u16::from_le_bytes(d[o..o + 2].try_into().unwrap())
}
fn u32_at(d: &[u8], o: usize) -> u32 {
    u32::from_le_bytes(d[o..o + 4].try_into().unwrap())
}
fn u64_at(d: &[u8], o: usize) -> u64 {
    u64::from_le_bytes(d[o..o + 8].try_into().unwrap())
}

fn off(name: &str) -> Result<usize, String> {
    let fleet = catalog()
        .account_type("Fleet")
        .ok_or("Fleet type missing")?;
    let f = fleet
        .fields
        .iter()
        .find(|f| f.name == name)
        .ok_or_else(|| format!("Fleet field {name} missing"))?;
    let o = f
        .offset
        .as_ref()
        .and_then(|v| v.as_u64())
        .ok_or_else(|| format!("Fleet field {name} has no offset"))?;
    Ok(o as usize)
}

pub fn decode_fleet_readout(data: &[u8]) -> Result<FleetReadout, String> {
    let label_at = off("fleetLabel")?;
    if data.len() < label_at + 64 {
        return Err("fleet too small".into());
    }
    let label = String::from_utf8_lossy(&data[label_at..label_at + 64])
        .trim_end_matches('\0')
        .to_string();
    let owner = off("ownerProfile")?;
    let sub = off("subProfile")?;
    Ok(FleetReadout {
        fleet_label: label,
        owner_profile: encode_pk(&data[owner..owner + 32]),
        sub_profile: encode_pk(&data[sub..sub + 32]),
        required_crew: u16_at(data, off("requiredCrew")?),
        passenger_capacity: u16_at(data, off("passengerCapacity")?),
        respawn_time_without_fee: u16_at(data, off("respawnTimeWithoutFee")?),
        crew_count: u16_at(data, off("crewCount")?),
        rented_crew: u16_at(data, off("rentedCrew")?),
        ap: u32_at(data, off("ap")?),
        fuel_id: u16_at(data, off("fuelTank.cargoId")?),
        fuel_amount: u64_at(data, off("fuelTank.amount")?),
        ammo_id: u16_at(data, off("ammoBank.cargoId")?),
        ammo_amount: u64_at(data, off("ammoBank.amount")?),
    })
}
