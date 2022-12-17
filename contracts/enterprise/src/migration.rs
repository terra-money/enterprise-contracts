use crate::state::{DAO_COUNCIL, DAO_METADATA, DAO_METADATA_KEY};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Storage;
use cw_storage_plus::Item;
use enterprise_protocol::api::{DaoMetadata, DaoSocialData, Logo};
use enterprise_protocol::error::DaoResult;

pub fn migrate_v1_to_v2(store: &mut dyn Storage) -> DaoResult<()> {
    DAO_COUNCIL.save(store, &None)?;

    migrate_dao_metadata_v1(store)?;

    Ok(())
}

#[cw_serde]
struct DaoMetadataV1 {
    pub name: String,
    pub logo: Logo,
    pub socials: DaoSocialData,
}

fn migrate_dao_metadata_v1(store: &mut dyn Storage) -> DaoResult<()> {
    let metadata_v1: DaoMetadataV1 = Item::new(DAO_METADATA_KEY).load(store)?;

    let metadata = DaoMetadata {
        name: metadata_v1.name,
        description: None,
        logo: metadata_v1.logo,
        socials: metadata_v1.socials,
    };
    DAO_METADATA.save(store, &metadata)?;

    Ok(())
}
