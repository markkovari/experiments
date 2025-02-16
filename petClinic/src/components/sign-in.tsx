import { component$ } from "@builder.io/qwik";
import { Form, Link } from "@builder.io/qwik-city";
import { useSignIn } from "../routes/plugin@auth";

const SignIn = component$(() => {
  const signInSig = useSignIn();

  return (
    <>
      <Form action={signInSig}>
        <input type="hidden" name="providerId" value="github" />
        <input type="hidden" name="options.redirectTo" value="/" />
        <button type="submit">Sign In</button>
      </Form>
    </>
  );
});

export { SignIn };
