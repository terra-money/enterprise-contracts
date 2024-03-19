import {env} from "process";

export const advanceTimeBy = async (millis: number) => {
    console.log(`Advancing chain time by ${millis} milliseconds.`)

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
        .then((response) => response.json())
        .then(() => console.log(`Chain time successfully advanced.`));
}