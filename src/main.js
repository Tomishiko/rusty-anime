const { app, BrowserWindow } = require('electron/main')

const createWindow = () => {
  const win = new BrowserWindow({
    width: 800,
    height: 600,
	title:process.argv[1],
	backgroundColor:'#000000',
	titleBarStyle: 'hidden',
	titleBarOverlay: {color:"#00000000",symbolColor: '#ffffff',}
  })
  let player = "./player/index.html"
  
  win.loadFile(player,{hash:process.argv[1]})
}

app.whenReady().then(() => {
  createWindow()

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow()
    }
  })
})

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit()
  }
})