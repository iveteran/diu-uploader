import { writable } from 'svelte/store';

export const baseUrl = "https://diu.matrixworks.cn"
export const uploadDir = 'diu_upload_documents'
//export const uploadUrl = `${baseUrl}/api/upload/file`
export const uploadUrl = 'http://localhost:8099/'
export const clientType = 3  // 0: undefined, 1: web, 2: client, 3: uploader

const userStr = localStorage.getItem("user")
console.log("userStr: ", userStr)
export const user = writable(userStr ? JSON.parse(userStr) : {
  name: undefined,
  id: 0,
  token: undefined,
})

user.subscribe((value) => {
  localStorage.setItem("user", JSON.stringify(value));
});
