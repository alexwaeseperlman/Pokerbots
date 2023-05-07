import React from "react";

export default function Login() {
  return (
    <>
      <a className="login-button" href="/api/login-provider?provider=microsoft">
        Log in with Microsoft
      </a>
    </>
  );
}
