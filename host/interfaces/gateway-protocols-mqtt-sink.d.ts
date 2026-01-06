/** @module Interface gateway:protocols/mqtt-sink **/
export function publish(topic: string, payload: string, qos: number): void;
export interface ErrorCode {
  code: number,
  message: string,
}
