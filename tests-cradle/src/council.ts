import {LCDClient} from "@terra-money/feather.js";
import {DaoContracts} from "./main";
import {queryGovConfig} from "./gov_controller_utils";


export type DaoCouncilMembers = {
    members: string[],
}

interface CouncilMembersResponse {
    members: CouncilMember[]
}

interface CouncilMember {
    user: string,
    weight: string,
}

export const queryDaoCouncil = async (lcd: LCDClient, dao: DaoContracts): Promise<DaoCouncilMembers> => {
    let response: CouncilMembersResponse;
    let startAfter: string | null = null;

    let members: string[] = [];

    do {
        response = await lcd.wasm.contractQuery<CouncilMembersResponse>(dao.council_membership_contract, {
            members: {
                start_after: startAfter,
                limit: 100,
            }
        });
        if (response.members.length > 0) {
            startAfter = response.members[response.members.length - 1].user;
        }

        members = [...members, ...response.members.map((member) => member.user)];
    } while (response.members.length > 0)

    return {
        members
    }
}

export const hasDaoCouncil = async (lcd: LCDClient, dao: DaoContracts): Promise<boolean> => {
    const govConfig = await queryGovConfig(lcd, dao);

    if (govConfig.council_gov_config === undefined) {
        return true;
    }

    const council = await queryDaoCouncil(lcd, dao);

    return council.members.length > 0
}