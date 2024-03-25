import {env} from "process";

const TWELVE_HOURS: number = 12 * 60 * 60 * 1000;

export const advanceTimeBy = async (millis: number) => {
    console.log(`Advancing chain time by ${millis} milliseconds.`)

    if (millis > TWELVE_HOURS) {
        let advancedBy = 0;
        while (advancedBy < millis) {
            // move by either the remaining time, or at most 12 hours at a time
            const nextStep = Math.min(millis - advancedBy, TWELVE_HOURS);

            await proposeBlock(nextStep);

            advancedBy += nextStep;
        }
    } else {
        await proposeBlock(millis)
    }
}

const proposeBlock = async (millis: number) => {
    const endpoint = env.LCD_ENDPOINT + '/next';
    const headers = {
        'Authorization': `Bearer ${env.JWT_TOKEN}`,
        'Content-Type': 'application/json'
    };

    const options = {
        method: 'POST',
        headers: headers,
        body: JSON.stringify({interval: millis})
    };

    await fetch(endpoint, options)
        .then((response) => response.json()); // TODO: add a success log here?
}