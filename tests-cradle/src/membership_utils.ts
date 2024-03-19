import {LCDClient} from "@terra-money/feather.js";
import {DaoContracts} from "./main";
import assert = require("node:assert");

export interface UserWeightResponse {
    user: string,
    weight: string,
}

export interface MembersResponse {
    members: UserWeightResponse[]
}

export const queryOneMember = async (lcd: LCDClient, dao: DaoContracts): Promise<UserWeightResponse> => {
    const response = await lcd.wasm.contractQuery<MembersResponse>(dao.membership_contract, {
        members: {
            limit: 1,
        }
    });

    if (response.members.length === 0) {
        throw new Error('No members found in the contract');
    }

    return response.members[0]
}

export const queryAllMembers = async (lcd: LCDClient, dao: DaoContracts): Promise<UserWeightResponse[]> => {
    let startAfter: string | null = null;

    let members: UserWeightResponse[] = [];

    let response: MembersResponse;

    do {
        response = await lcd.wasm.contractQuery<MembersResponse>(dao.membership_contract, {
            members: {
                start_after: startAfter,
                limit: 100,
            }
        });
        members = [...members, ...response.members];
        if (response.members.length > 0) {
            startAfter = response.members[response.members.length - 1].user;
        }
    } while (response.members.length > 0);

    return members
}

export const queryUserWeight = async (lcd: LCDClient, dao: DaoContracts, user: string): Promise<number> => {
    const response = await lcd.wasm.contractQuery<UserWeightResponse>(dao.membership_contract, {
        user_weight: {
            user,
        }
    });
    return parseInt(response.weight, 10)
}

export const assertUserWeight = async (lcd: LCDClient, dao: DaoContracts, user: string, weight: number) => {
    const userWeight = await queryUserWeight(lcd, dao, user);

    assert(userWeight === weight, "User weight was different from expected");
}