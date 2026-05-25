import { Pool } from "pg";

export const pool = new Pool({
  connectionString:
    import.meta.env.DATABASE_URL ??
    "postgresql://inamute:inamute_password@localhost:5432/inamute"
});
