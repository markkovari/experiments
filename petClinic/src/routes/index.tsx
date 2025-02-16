import { component$ } from "@builder.io/qwik";
import { type DocumentHead, Form } from "@builder.io/qwik-city";
import { SignIn } from "~/components/sign-in";
import { useSession, useSignIn, useSignOut } from "./plugin@auth";

export default component$(() => {
  const signIn = useSignIn();
  const { value: user } = useSession();
  const signOut = useSignOut();

  return (
    <>
      {user ? (
        <>
          <h1>Welcome, {user.user?.name}!</h1>
          <Form action={signOut}>
            <button type="button">Sign Out</button>
          </Form>
        </>
      ) : (
        <SignIn />
      )}
    </>
  );
});

export const head: DocumentHead = {
  title: "Welcome to Qwik",
  meta: [
    {
      name: "description",
      content: "Qwik site description",
    },
  ],
};
