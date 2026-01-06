/** @module Interface gateway:protocols/metrics **/
export function getStats(): GatewayStats;
export interface GatewayStats {
  framesProcessed: bigint,
  framesInvalid: bigint,
  bytesIn: bigint,
  bytesOut: bigint,
  lastError?: string,
}
