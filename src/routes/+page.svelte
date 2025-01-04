<script>
  import { onMount, onDestroy } from 'svelte'
  import { invoke } from "@tauri-apps/api/core";
  import { open, Command } from '@tauri-apps/plugin-shell';
  import { openUrl, openPath } from '@tauri-apps/plugin-opener'
  import { goto } from '$app/navigation';
  import { stat, readTextFile, readDir, BaseDirectory, watch } from '@tauri-apps/plugin-fs';
  import { join, homeDir } from '@tauri-apps/api/path';
  import { user, baseUrl, uploadDir, uploadUrl } from '../stores.js';

  let _name = $state("")
  let _greetMsg = $state("")
  let _files = $state([])

  const _status_filters = [
    { label: 'All', value: 0 },
    { label: 'Uploading', value: 1 },
    { label: 'Uploaded', value: 2 },
    { label: 'Failed', value: 3 },
  ];
  let _status_selected = $state(_status_filters[1])

  const _filetype_filters = [
    { label: 'All', value: 0 },
    { label: 'CSV', value: 1 },
    { label: 'Excel', value: 2 },
    { label: 'Markdown', value: 3 },
    { label: 'PDF', value: 4 },
  ];
  let _filetype_selected = $state(_filetype_filters[0])

  let pathRoot = $state("")
  let baseDir = $state("")

  onMount(async () => {
    console.log("> onMount begin")

    console.log("> loading user: ", $user)
    if (! $user || ! $user.token) {
      login()
    }

    await listFiles()
    console.log("_files:", $state.snapshot(_files))
    console.log("> onMount done")

    pathRoot = await homeDir()
    baseDir = `${pathRoot}/${uploadDir}`

    await enableWatchFiles()
  })

  async function greet(event) {
    event.preventDefault();
    _greetMsg = await invoke("greet", { _name });
  }

  async function listFiles() {
    const dir = uploadDir
    console.log("list _files of directory: ", dir)
    const entries = await readDir(dir, { baseDir: BaseDirectory.Home });
    await processEntriesRecursively(dir, entries, _files);
  }

  async function processEntriesRecursively(parent, entries, _files) {
    for (const entry of entries) {
      console.log('> file entry: ', entry);
      const path = await join(parent, entry.name);
      const fileType = entry.name.split('.').pop().toLowerCase();
      console.log("> file type: ", fileType)
      if (entry.isDirectory) {
        const subEntries = await readDir(path, { baseDir: BaseDirectory.Home })
        await processEntriesRecursively(path, subEntries, _files)
      } else {
        const metadata = await stat(path, {
          baseDir: BaseDirectory.Home,
        });
        console.log("> file metadata: ", metadata)
        _files.push({
          path: path,
          size: metadata.size,
          //size: metadata.size * 1024 * 1000,  // just for test to show value with unit
          mtime: metadata.mtime,
          file_type: fileType,
        })
      }
    }
  }

  async function reloadFiles() {
     await listFiles()
  }

  async function uploadFiles() {
    for (const file of _files) {
      const filePath = `${pathRoot}/${file}`

      const fileContent = await readTextFile(filePath, {
        baseDir: BaseDirectory.Home,
      });                                                                   
      console.log("> file content:", fileContent)

      console.log("> Uploading file: ", filePath)

      await uploadFile(filePath)
    }
    await test_upload()
  }

  async function uploadFile(filePath) {
  }

  async function test_upload() {
    invoke('my_upload', {
      url: uploadUrl,
      headers: {
        'x-token': 'my_token',
      },
      //formParams: {
      //  'lang': 'zh-CN',
      //  'count': '100',
      //},
      formParams: {},
      formKvs: {
        'my_key': 'my_value',
        'my_key_2': 'my_value_2',
      },
      formFiles: {
        'files': [`${baseDir}/hello_world.txt`, `${baseDir}/hello.txt`],
      },
      body: '',
      //body: 'foo bar',
    }).then((value) => {
      console.log("test_upload success: ", value)
    }).catch((err) => {
      console.error("test_upload failed: ", err)
    })
  }

  async function stopUpload(index) {
  }

  async function login() {
    goto('/login')
  }

  async function logout() {
    $user.token = undefined
    goto('/login')  // redirect to login page
  }

  async function open_console_by_browser() {
    //await open(baseUrl)
    await openUrl(baseUrl)  // Refer: https://lib.rs/crates/tauri-plugin-opener
  }

  async function open_console_by_client() {
    // XXX: Need to configure permission in src-tauri/capabilities/default.json
    let result = await Command.create('DIU').execute();
    console.log("Executing DIU client result:", result);
  }

  async function openDirectory() {
    await openPath(baseDir)
    //await openPath(baseDir)
  }

  async function enableWatchFiles() {
    await watch(
      uploadDir,
      (event) => {
        console.log('DIU documents upload directory event', event);
      },
      {
        baseDir: BaseDirectory.Home,
        recursive: true,
      }
    )
  }

  function sizeWithUnit(value) {
    const GB = 1024 * 1024 * 1024
    const MB = 1024 * 1024
    const KB = 1024
    if (value >= GB) {
      const gb_value = (value / GB).toFixed(2)
      return `${gb_value} GB`
    } else if (value >= MB) {
      const mb_value = (value / MB).toFixed(2)
      return `${mb_value} MB`
    } else if (value >= KB) {
      const kb_value = (value / KB).toFixed(2)
      return `${kb_value} KB`
    } else {
      return value
    }
  }

</script>

<main class="container">
  <nav>
    <div class="title">DIU Documents Uploader</div>
    <div class="row settings">
      {#if $user && $user.token}
        <button class="btn-small btn-primary" onclick={open_console_by_browser}>Console (Browser)</button>
        <button class="btn-small btn-primary" onclick={open_console_by_client}>Console (Client)</button>
        <button class="btn-small btn-primary" onclick={logout}>Logout</button>
      {:else}
        <button class="btn-small btn-primary" onclick={login}>Login</button>
      {/if}
    </div>
  </nav>

  <div class="main-container">
    <div class="row">
      <span>Documents directory: { baseDir }</span>
      <button class="btn-small btn-primary" onclick={openDirectory}>Open in file explorer</button>
    </div>

    <div class="row">
      {#each _status_filters as value}
        <label>
          <input
            type="radio"
            bind:group={_status_selected}
            name="status_filter"
            value={value}
            > {value.label}
        </label>
      {/each}
    </div>

    <div class="row">
      {#each _filetype_filters as value}
        <label>
          <input
            type="radio"
            bind:group={_filetype_selected}
            name="filetype_filter"
            value={value}
            > {value.label}
        </label>
      {/each}
    </div>

    <table cellspacing="10" cellpadding="10">
      <thead>
        <tr>
          <th>Index</th>
          <th>File</th>
          <th>Size</th>
          <th>Last Modify</th>
          <th>Status</th>
          <th>
          <button class="btn-small btn-warning" onclick={stopUpload}>Stop All</button>
          </th>
        </tr>
      </thead>
      <tbody>
        {#each _files as file, index}
          <tr>
            <td>{ index + 1}</td>
            <td>{ file['path'].substring(file['path'].indexOf('/')) }</td>
            <td>{ sizeWithUnit(file['size']) }</td>
            <td>{ file['mtime'].toISOString().split('.')[0] }</td>
            <td>c</td>
            <td>
              <button class="btn-small btn-warning" onclick={stopUpload(index)}>Stop</button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>

</main>

<style>
  @import "../app.css";
</style>
