import {LCDClient} from "@terra-money/feather.js";
import {ComponentContractsResponse} from "./types/enterprise";
import {DaoContracts} from "./main";

export const getDaoComponents = async (lcd: LCDClient, daoEnterprise: string): Promise<DaoContracts> => {
    const componentContracts = await lcd.wasm.contractQuery<ComponentContractsResponse>(daoEnterprise, {
        component_contracts: {}
    });
    return { ...componentContracts, enterprise: daoEnterprise }
}