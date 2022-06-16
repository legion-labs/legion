// Resolves the Cognito issuer url based on the region and the pool id
export function resolveIssuerUrl({
  region,
  poolId,
}: {
  region: string;
  poolId: string;
}) {
  return `https://cognito-idp.${region}.amazonaws.com/${poolId}`;
}
