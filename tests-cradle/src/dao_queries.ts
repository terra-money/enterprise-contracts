// TODO: rename this file

import {LCDClient} from "@terra-money/feather.js";
import {DaoContracts} from "./main";
import {DaoInfoResponse} from "./types/enterprise";

export const queryDaoInfo = async (lcd: LCDClient, dao: DaoContracts): Promise<DaoInfoResponse> => {
    return lcd.wasm.contractQuery<DaoInfoResponse>(dao.enterprise, {
        dao_info: {}
    });
}

export const isNotDaoContract = (dao: DaoContracts, address: string): boolean => {
    return address !== dao.attestation_contract
        && address !== dao.council_membership_contract
        && address !== dao.enterprise_factory_contract
        && address !== dao.enterprise_governance_contract
        && address !== dao.enterprise_governance_controller_contract
        && address !== dao.enterprise_outposts_contract
        && address !== dao.enterprise_treasury_contract
        && address !== dao.funds_distributor_contract
        && address !== dao.membership_contract
}

export const isNotContract = async (lcd: LCDClient, address: string): Promise<boolean> => {
    try {
        await lcd.wasm.contractInfo(address);
        return false
    } catch (e) {
        // TODO: what if it fails for other reasons?
        return true
    }
}