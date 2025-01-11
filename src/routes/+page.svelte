<script>
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from "@tauri-apps/api/core";
  import { fetch } from '@tauri-apps/plugin-http';
  import { open, Command } from '@tauri-apps/plugin-shell';
  import { openUrl, openPath } from '@tauri-apps/plugin-opener';
  import { goto } from '$app/navigation';
  import { stat, readTextFile, readDir, BaseDirectory, watch } from '@tauri-apps/plugin-fs';
  import { md5 } from 'js-md5';  // https://github.com/emn178/js-md5
  import { join, homeDir } from '@tauri-apps/api/path';
  import { user, baseUrl, uploadDir, uploadUrl, clientType } from '../stores.js';

  let _errorMsg = $state("");
  let _name = $state("")
  let _greetMsg = $state("")
  let _files = $state([])
  let _uploadQueue = $state([])
  let _uploadedFiles = $state({})
  let _uploadedFileTotal = 0
  let _running = $state(false)
  //let _removingQueue = $state([])

  const Status = Object.freeze({
    All: { name: "All", value: -1 },
    Unprocessed: { name: "Unprocessed", value: 0 },
    InQueue:   { name: "In Queue", value: 1 },
    Uploading: { name: "Uploading", value: 2 },
    Uploaded:  { name: "Uploaded", value: 3 },
    Failed:    { name: "Failed", value: 4 },
    Cancelled: { name: "Cancelled", value: 5 },
  });

  const _supportedFileTypes = new Set([
    'txt',
    'csv',
    'xls',
    'xlsx',
    'pdf',
    'md',
  ])
  const _status_filters = [
    { label: Status.All.name, value: Status.All },
    { label: Status.Unprocessed.name, value: Status.Unprocessed },
    { label: Status.InQueue.name, value: Status.InQueue },
    { label: Status.Uploading.name, value: Status.Uploading },
    { label: Status.Uploaded.name, value: Status.Uploaded },
    { label: Status.Failed.name, value: Status.Failed },
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

  function getStatuName(value) {
    for (const status of Object.values(Status)) {
      if (status.value === value) {
        return status.name
      }
    }
    return "Unknown"
  }

  onMount(async () => {
    console.log("> onMount begin")

    console.log("> loading user: ", $user)
    if (! $user || ! $user.token) {
      login()
    }

    pathRoot = await homeDir()
    baseDir = `${pathRoot}/${uploadDir}`

    await listFiles()
    console.log("_files:", $state.snapshot(_files))

    await getUploadedFiles()
    console.log("_uploadedFiles:", $state.snapshot(_uploadedFiles))

    enqueueUnuploadedFiles()

    //await enableWatchFiles()
    console.log("> onMount done")
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
      if (entry.isDirectory) {
        const subEntries = await readDir(path, { baseDir: BaseDirectory.Home })
        await processEntriesRecursively(path, subEntries, _files)
      } else {
        const fileObj = await buildFileObject(path)
        _files.push(fileObj)
      }
    }
  }

  function getFileObjectByPath(filePath) {
    return _files.find((item) => item['path'] === filePath)
  }

  function removeLocalFile(file) {
    _files = _files.filter((item) => { return item['md5'] !== file['md5'] })
    console.log("Removed item of _files: ", $state.snapshot(_files))
  }

  async function buildFileObject(path) {
    const fileType = path.split('.').pop().toLowerCase();
    console.log("> file type: ", fileType)

    const metadata = await stat(path, {
      baseDir: BaseDirectory.Home,
    });
    console.log("> file metadata: ", metadata)

    const fileContent = await readTextFile(path, {
      baseDir: BaseDirectory.Home,
    });
    //console.log("> file content:", fileContent)
    let hash = md5.create()
    hash.update(fileContent)
    const fileMd5 = hash.hex()
    console.log("> file md5:", fileMd5)

    return {
      path: path,
      size: metadata.size,
      //size: metadata.size * 1024 * 1000,  // just for test to show value with unit
      mtime: metadata.mtime,
      file_type: fileType,
      md5: fileMd5,
      status: Status.Unprocessed.value,
    }
  }

  async function reloadFiles() {
     await listFiles()
  }

  async function uploadFile(file) {
    file['status'] = Status.Uploading.value  // uploading

    const filePath = file['path']
    const fileFullPath = `${pathRoot}/${filePath}`
    console.log("> Uploading file: ", fileFullPath)

    invoke('my_upload', {
      url: uploadUrl,
      //url: 'http://localhost:8099/',  // just for debug
      headers: {
        'x-uid': String($user.id),
        'x-token': $user.token,
        'x-client': clientType,
        'mwx-ua': clientType,  // Compatible to old version
      },
      formKvs: {},      // also must be there if it is empty
      formParams: {},   // also must be there if it is empty
      formFiles: {
        'files': [fileFullPath],
      },
      body: '',   // also must be there if it is empty
    }).then((value) => {
      file['status'] = Status.Uploaded.value  // uploaded
      removeUpload(file)
      console.log("uploadFile success: ", value)
    }).catch((err) => {
      file['status'] = Status.Failed.value  // failed
      removeUpload(file)
      console.error("uploadFile failed: ", err)
    })
  }

  function removeUpload(file) {
    _uploadQueue = _uploadQueue.filter((item) => { return item['md5'] !== file['md5'] })
    console.log("Removed item from _uploadQueue: ", $state.snapshot(_uploadQueue))
  }

  function removeUploadByPath(filePath) {
    _uploadQueue = _uploadQueue.filter((item) => { return item['path'] !== file['path'] })
    console.log("Removed item from _uploadQueue: ", $state.snapshot(_uploadQueue))
  }

  function existsInUploadQueue(file_md5) {
    return _uploadQueue.filter((item) => { return item['md5'] === file_md5 })
  }

  function enqueueUpload(file) {
    file['status'] = Status.InQueue.value  // pending
    if (! existsInUploadQueue(file['md5'])) {
      _uploadQueue.push(file)
      console.log("Added item to _uploadQueue: ", $state.snapshot(_uploadQueue))
    }
  }

  function dequeueUpload(file) {
    file['status'] = Status.Cancelled.value  // cancelled
    removeUpload(file)
    //_removingQueue.remove(fileRelativePath)
  }

  function dequeueUploadByPath(filePath) {
    removeUploadByPath(filePath)
  }

  function enqueueUnuploadedFiles() {
    for (const file of _files) {
      if (file.md5 in _uploadedFiles) {
        file.status = Status.Uploaded.value
      } else {
        enqueueUpload(file)
      }
    }
  }

  function stopUpload() {
    _running = false
  }

  async function startUpload() {
    _running = true
    await runUpload()
  }

  async function runUpload() {
    //await test_upload()

    while (_running) {
      let file = _uploadQueue.shift()
      if (!file) {
        break
      }
      await uploadFile(file)
    }
    _running = false
  }

  async function getUploadedFiles() {
    const response = await fetch(`${baseUrl}/api/upload/file/list`, {
    //const response = await fetch('http://localhost:8099/', {
      method: 'GET',
      timeout: 10,  // seconds
      headers: {
        "x-uid": $user.id,
        "x-token": $user.token,
        'x-client': clientType,
        'mwx-ua': clientType,  // Compatible to old version
      },
    })

    response.json().then((result) => {
      console.log('get uploaded files result: ', result)
      if (result.code === 0) {
        //_uploadedFiles = result.data.items
        for (const item of result.data.items) {
          _uploadedFiles[item['file_id']] = item
        }
        _uploadedFileTotal = result.data.total
      } else {
        console.error('login err: ', result.message)
        _errorMsg = result.message
      }
    }).catch((err) => {
      console.error('login exception: ', err)
      _errorMsg = err
    })
  }

  async function removeServerUploadedFile(fileId) {
    console.log('removing uploaded file: ', fileId)
    const response = await fetch(`${baseUrl}/api/upload/file?fileid=${fileId}`, {
    //const response = await fetch('http://localhost:8099/', {
      method: 'DELETE',
      timeout: 10,  // seconds
      headers: {
        "x-uid": $user.id,
        "x-token": $user.token,
        'x-client': clientType,
        'mwx-ua': clientType,  // Compatible to old version
      },
    })

    response.json().then((result) => {
      console.log('removing uploaded file result: ', result)
      if (result.code === 0) {
        console.info('removed file from server, success: ', fileId)
      } else {
        console.error('removing file from server, err: ', result.message)
        _errorMsg = result.message
        // TODO: add to operation log and trying late
      }
    }).catch((err) => {
      console.error('removing file, exception: ', err)
      _errorMsg = err
      // TODO: add to operation log and trying late
    })
  }

  async function test_upload() {
    invoke('my_upload', {
      url: uploadUrl,
      //url: 'http://localhost:8099/',  // just for debug
      headers: {
        'x-token': 'my_token',
        'x-client': clientType,
        'mwx-ua': clientType,  // Compatible to old version
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

  async function login() {
    goto('/login')
  }

  async function logout() {
    $user.token = undefined
    goto('/login')  // redirect to login page
  }

  async function open_console_by_browser() {
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
      {#each _status_filters as status}
        <label>
          <input
            type="radio"
            bind:group={_status_selected}
            name="status_filter"
            value={status}
            > {status.label}
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

    {#if _errorMsg}
      <p id="err-msg">{_errorMsg}</p>
    {/if}

    <table cellspacing="10" cellpadding="10">
      <thead>
        <tr>
          <th>Index</th>
          <th>File</th>
          <th>Size</th>
          <th>Last Modification</th>
          <th>Status</th>
          <th>
            {#if _running}
              <button class="btn-small btn-warning" onclick={stopUpload}>Stop</button>
            {:else if (_uploadQueue.length > 0)}
              <button class="btn-small btn-primary" onclick={startUpload}>start</button>
            {/if}
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
            <td>{ getStatuName(file['status']) }</td>
            <td>
              {#if (file['status'] === Status.Unprocessed.value ||
                file['status'] === Status.Failed.value ||
                file['status'] === Status.Cancelled.value)}
                <button class="btn-small btn-primary" onclick={() => enqueueUpload(file)}>Upload</button>
              {:else if (file['status'] === Status.InQueue.value) }
                <button class="btn-small btn-warning" onclick={() => dequeueUpload(file)}>Cancel</button>
              {:else }
              {/if}
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
