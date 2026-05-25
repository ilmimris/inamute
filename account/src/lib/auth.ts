import { betterAuth } from "better-auth";
import { pool } from "./db";

const baseURL = import.meta.env.BETTER_AUTH_URL ?? "http://localhost:4321";
const googleClientId = import.meta.env.GOOGLE_CLIENT_ID;
const googleClientSecret = import.meta.env.GOOGLE_CLIENT_SECRET;

export const auth = betterAuth({
  appName: "Inamute",
  baseURL,
  database: pool,
  trustedOrigins: [baseURL],
  socialProviders: {
    google: {
      clientId: googleClientId ?? "",
      clientSecret: googleClientSecret ?? "",
      prompt: "select_account"
    }
  }
});

export const hasGoogleAuthConfig = Boolean(googleClientId && googleClientSecret);
