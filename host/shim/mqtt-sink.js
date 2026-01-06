// host/shim/mqtt-sink.js
// mock mqtt sink that captures published payloads.
// provides the mqtt-sink interface to the wasm guest.
// stores all published messages for testing and dashboard display.

// storage for published messages
let publishedMessages = [];
let publishCallback = null;

/**
 * register a callback to be called on each publish
 * useful for real-time dashboard updates
 * @param {function} callback - receives (topic, payload, qos)
 */
export function onPublish(callback) {
    publishCallback = callback;
}

/**
 * get all published messages
 * @returns {Array} array of {topic, payload, qos, timestamp} objects
 */
export function getPublishedMessages() {
    return [...publishedMessages];
}

/**
 * clear all stored messages
 */
export function clearMessages() {
    publishedMessages = [];
}

/**
 * wit interface implementation: publish
 * stores the message and calls any registered callback
 * @param {string} topic - mqtt topic
 * @param {string} payload - json payload string
 * @param {number} qos - quality of service (0, 1, or 2)
 */
export function publish(topic, payload, qos) {
    const message = {
        topic,
        payload,
        qos,
        timestamp: new Date().toISOString(),
    };

    publishedMessages.push(message);

    // log for demo visibility
    console.log(`[MQTT-SINK] Published to ${topic}: ${payload.substring(0, 50)}...`);

    // call registered callback if any
    if (publishCallback) {
        publishCallback(topic, payload, qos);
    }

    // return success
    return { tag: 'ok', val: undefined };
}
