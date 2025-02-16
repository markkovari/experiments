import { component$ } from "@builder.io/qwik";
import { Form, Link } from "@builder.io/qwik-city";
import { useSignIn } from "../routes/plugin@auth";

const SignIn = component$(() => {
  const signInSig = useSignIn();

  return (
    <>
      {/* server-side login with Form action */}
      <Form action={signInSig}>
        <input type="hidden" name="providerId" value="github" />
        <input type="hidden" name="options.redirectTo" value="/" />
        <button>Sign In</button>
      </Form>

      {/* submit method */}
      <Link onClick$={() => signInSig.submit({ redirectTo: "/" })}>SignIn</Link>
    </>
  );
});

export { SignIn };
