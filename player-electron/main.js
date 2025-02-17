const { app, BrowserWindow,ipcMain } = require('electron/main')
const { session } = require('electron')
const path = require("path")
const { traceProcessWarnings } = require('process')

let win
const createWindow = () => {
  win = new BrowserWindow({
    width: 840,
    height: 515,
	  title:"Aniplayer",
    backgroundColor:'#000000',
    titleBarStyle: 'hidden',
    titleBarOverlay: { color: "#00000000", symbolColor: '#ffffff', },
    // webPreferences: {
    //   nodeIntegration: true,
    //   contextIsolation: true,
    //   preload: path.join(__dirname, 'preload.js')
    // }
  })

  // let info = [
  //   process.argv[2], // chapter points
  //   process.argv[3], // thumbnail
  //   process.argv[4]  // name
  // ]

  //const sessid1 = { url: 'https://www.anilibria.tv', domain: 'www.anilibria.tv', path: '/', name: 'PHPSESSID', value: 'NUxA5YdDlWnnMPnT0oZpu6e7ZO8Wnrnu' }
  //const sessid2 = { url: 'https://www.anilibria.tv', domain: '.anilibria.tv', path: '/', name: 'PHPSESSID', value: 'NUxA5YdDlWnnMPnT0oZpu6e7ZO8Wnrnu' }

  
// session.defaultSession.cookies.set(sessid1)
//   .then(() => {
//     // success
//   }, (error) => {
//     console.error(error)
//   })
//   session.defaultSession.cookies.set(sessid2)
//   .then(() => {
//     // success
//   }, (error) => {
//     console.error(error)
//   })
  let player = "./player/index.html"
  
  //win.loadFile(player, { hash: process.argv[1] })
  
  win.loadFile(player, {
    query: {
      //"chapters": process.argv[2],
      "timings": process.argv[1],
      "source": process.argv[3],
      "title": process.argv[2],
     }
  })
    .then(() => { win.show(); })
  //win.webContents.openDevTools()
  //console.log(process.argv);
}

app.whenReady().then(() => {
  createWindow()
})

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit()
  }
})