<script>
  let rock = "\u{1F44A}";
  let paper = "\u{1F590}";
  let scissors = "\u{270C}";

  let connected = false;

  let choice = "";

  let websocket = null;
  let log = [];

  /**
   * @param {string} value
   */
  function selectChoice(value) {
    // choice = value;
    // console.log(value);
    websocket.send(value);
  }

  function connectToWebsocket() {
    websocket = new WebSocket("ws://localhost:3000/websocket");
    websocket.onopen = function () {
      console.log("[CLIENT]: connection opened");
    };

    websocket.onclose = function () {
      console.log("[CLIENT]: connection closed");
      // btn.disabled = false;
    };

    connected = true;

    websocket.onmessage = function (e) {
      console.log("[CLIENT]: received message: " + e.data);
      log = [...log, e.data + "\n"];
    };

    // input.onkeydown = function (e) {
    //   if (e.key == "Enter") {
    //     websocket.send(input.value);
    //     input.value = "";
    //   }
    // };
  }
</script>

<main>
  <h1>RPS</h1>
  {#each log as message}
    <p>{message}</p>
  {/each}

  {#if connected}
    <div id="icon-container">
      <button type="button" on:click={() => selectChoice("rock")} class="icon"
        >{rock}</button
      >
      <button type="button" on:click={() => selectChoice("paper")} class="icon"
        >{paper}</button
      >
      <button
        type="button"
        on:click={() => selectChoice("scissors")}
        class="icon">{scissors}</button
      >
    </div>
  {:else}
    <button on:click={connectToWebsocket}>Connect</button>
  {/if}
</main>

<style>
  .icon:hover {
    filter: drop-shadow(0 0 0.2em #646cffaa);
    cursor: pointer;
    transform: scale(1.2);
  }
  .icon {
    /* color: #888; */
    font-size: 75px;
    color: white;
    background-color: rgb(64, 64, 255);
    padding: 25px;
    border-radius: 15px;
    transition: filter 300ms;
    transition: transform 0.2s;
  }

  button {
    color: white;
    font-size: 25px;
    background-color: rgb(64, 64, 255);
    margin-top: 50px;
  }

  #icon-container {
    display: flex;
    width: 700px;
    justify-content: space-evenly;
  }
</style>
