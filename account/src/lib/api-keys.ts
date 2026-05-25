import { createHmac, randomBytes, randomUUID } from "node:crypto";
import { pool } from "./db";

const API_KEY_PREFIX = "inamute_live_";
const MIN_API_KEY_HASH_SECRET_LEN = 32;

export type AccountApiKey = {
  id: string;
  keyPrefix: string;
  name: string;
  rateLimitRpm: number;
  isActive: boolean;
  createdAt: Date;
  lastUsedAt: Date | null;
  expiresAt: Date | null;
};

export function generateApiKey() {
  return `${API_KEY_PREFIX}${randomBytes(24).toString("base64url")}`;
}

function getApiKeyHashSecret() {
  const secret = import.meta.env.API_KEY_HASH_SECRET;

  if (!secret || secret.length < MIN_API_KEY_HASH_SECRET_LEN) {
    throw new Error(
      `API_KEY_HASH_SECRET must be at least ${MIN_API_KEY_HASH_SECRET_LEN} characters and shared with the Actix API`
    );
  }

  return secret;
}

export function hashApiKey(apiKey: string) {
  return createHmac("sha256", getApiKeyHashSecret()).update(apiKey).digest("hex");
}

export async function createApiKey(userId: string, name: string) {
  const apiKey = generateApiKey();
  const keyHash = hashApiKey(apiKey);
  const keyPrefix = apiKey.slice(0, API_KEY_PREFIX.length + 6);
  const trimmedName = name.trim() || "Default key";

  await pool.query(
    `INSERT INTO account_api_keys
      (id, user_id, key_hash, key_prefix, name, rate_limit_rpm)
     VALUES ($1, $2, $3, $4, $5, 60)`,
    [randomUUID(), userId, keyHash, keyPrefix, trimmedName]
  );

  return apiKey;
}

export async function listApiKeys(userId: string): Promise<AccountApiKey[]> {
  const result = await pool.query(
    `SELECT id, key_prefix, name, rate_limit_rpm, is_active, created_at, last_used_at, expires_at
     FROM account_api_keys
     WHERE user_id = $1
     ORDER BY created_at DESC`,
    [userId]
  );

  return result.rows.map((row) => ({
    id: row.id,
    keyPrefix: row.key_prefix,
    name: row.name,
    rateLimitRpm: row.rate_limit_rpm,
    isActive: row.is_active,
    createdAt: row.created_at,
    lastUsedAt: row.last_used_at,
    expiresAt: row.expires_at
  }));
}

export async function revokeApiKey(userId: string, keyId: string) {
  await pool.query(
    `UPDATE account_api_keys
     SET is_active = false
     WHERE user_id = $1 AND id = $2`,
    [userId, keyId]
  );
}
