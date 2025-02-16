import { component$ } from "@builder.io/qwik";
import { Form, type DocumentHead } from "@builder.io/qwik-city";
import { useSession, useSignIn, useSignOut } from "./plugin@auth";

export default component$(() => {
  const signIn = useSignIn();
  const { value: user } = useSession();
  const signOut = useSignOut();

  return (
    <>
      <div role="presentation" class="ellipsis"></div>
      {user ? (
        <>
          <h1>Welcome, {user.user?.name}!</h1>
          <Form action={signOut}>
            <button>Sign Out</button>
          </Form>
        </>
      ) : (
        <Form action={signIn}>
          <input type="hidden" name="providerId" value="github" />
          <input
            type="hidden"
            name="options.redirectTo"
            value="http://localhost:3000/auth/discord/callback"
          />
          <button>Sign In</button>
        </Form>
      )}
      <div role="presentation" class="ellipsis ellipsis-purple"></div>
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
