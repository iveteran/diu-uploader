<script>
  import { invoke } from "@tauri-apps/api/core";
  import { openUrl } from '@tauri-apps/plugin-opener'
  import { fetch } from '@tauri-apps/plugin-http';
  import { goto } from '$app/navigation';
  import { user, baseUrl, clientType } from '../../stores.js';

  let _username = $state("");
  let _password = $state("");
  let _errorMsg = $state("");

  console.log("> login page, user: ", $user)

  async function login(event) {
    event.preventDefault();

    // Send a request
    //const response = await fetch('http://localhost:8099/', {  // for test
    const response = await fetch(`${baseUrl}/api/user/login`, {
      method: 'POST',
      timeout: 10,  // seconds
      headers: {"content-type": "application/json"},
      body: JSON.stringify({
        username: _username,
        password: _password,
        client_type: clientType,
      }),
    })

    response.json().then((result) => {
      console.log('login result: ', result)
      if (result.code === 0) {
        const gotUser = result.data
        $user.token = gotUser.token
        $user.id = gotUser.uid
        $user.name = _username 

        goto('/');
      } else {
        console.log('login err: ', result.message)
        _errorMsg = result.message;
      }
    }).catch((err) => {
      console.log('login exception: ', err)
      _errorMsg = err
    })
  }

  async function sign_up() {
    // opens the given URL on the default browser:
    await openUrl(`${baseUrl}/#/register`);
  }

  async function retrieve_password() {
    await openUrl(`${baseUrl}/#/retrieve_password`);
  }

</script>

<main class="container">
  <nav>
    <div class="title">DIU Documents Uploader</div>
  </nav>

  <div class="main-container">
    <form class="column" onsubmit={login}>
      <input id="username" placeholder="Username" bind:value={_username} />
      <input id="password" type="password" placeholder="Password" bind:value={_password} />
      <button class="btn-primary" style="width: 100%;" type="submit">Login</button>
    </form>
    <div class="row">
      <button onclick={retrieve_password}>Retrieve Password</button>
      <button onclick={sign_up}>Sign Up</button>
    </div>
    <p id="err-msg">{_errorMsg}</p>
  </div>

</main>

<style>
  @import "../../app.css";
</style>
