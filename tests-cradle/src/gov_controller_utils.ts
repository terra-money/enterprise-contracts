import {DaoContracts} from "./main";
import {GovConfigResponse} from "./types/enterprise_governance_controller";
import {LCDClient} from "@terra-money/feather.js";

export const queryGovConfig = async (lcd: LCDClient, dao: DaoContracts): Promise<GovConfigResponse> => {
    return lcd.wasm.contractQuery<GovConfigResponse>(dao.enterprise_governance_controller_contract, {
        gov_config: {}
    });
}