import { QwikAuth$ } from "@auth/qwik";
import Discord from "@auth/qwik/providers/discord";

export const { onRequest, useSession, useSignIn, useSignOut } = QwikAuth$(
  () => ({
    providers: [Discord],
  }),
);
