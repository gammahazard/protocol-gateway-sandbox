/** @module Interface gateway:protocols/modbus-source **/
export function receiveFrame(): Uint8Array;
export interface ErrorCode {
  code: number,
  message: string,
}
