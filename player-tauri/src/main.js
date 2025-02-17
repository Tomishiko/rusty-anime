const { invoke } = window.__TAURI__.core;
const { emit, listen } = window.__TAURI__.event;
const { Window } = window.__TAURI__.window;
//const { Process } = window.__TAURI__.process;
let video;
let player;
let title;
const appWindow = new Window('main');


document.addEventListener("fullscreenchange", () => {
	if (document.fullscreenElement) {
		appWindow.setFullscreen(true);
	} else {
		appWindow.setFullscreen(false);
	}
});
window.addEventListener("DOMContentLoaded", function () {
	video = this.document.querySelector('video');
	title = this.document.getElementById('titleBar');
	invoke("read_streamed_data").then((val) => {
		let data = JSON.parse(val);
		init_player(data);
	});


});
const unlisten = await listen('reload_event', async function (e) {
	console.log('reload request from backend');
	location.reload();

})

function process_skips(skips) {
	let skipPoints;
	try {

		if (skips.length > 2) {
			skipPoints = [
				{ time: skips[0], label: 'Opening start' },
				{ time: skips[1], label: 'Opening end' },
				{ time: skips[2], label: 'Ending start' },
				{ time: skips[3], label: 'Ending End' },
			]
		}
		else {
			skipPoints = [
				{ time: skips[0], label: 'Opening start' },
				{ time: skips[1], label: 'Opening end' },
			]
		}
		return skipPoints;
	}
	catch (e) {
		console.log("Skip points were not provided");
		return null;
	}
}
function init_player(data) {
	let skipPoints = process_skips(data.skips)
	player = new Plyr(video, {
		markers: {
			enabled: true,
			points: skipPoints,
		},
		keyboard: { focused: true, global: true },
	});

	player.on('controlshidden', (event) => {
		title.classList.add('hidden');
	});

	player.on('controlsshown', (event) => {
		title.classList.remove('hidden');
	});
	document.getElementById("titleText").innerHTML = data.episode_title;

	appWindow.setTitle(data.title);
	hls_init(data.source);
}
function hls_init(source) {

	if (!Hls.isSupported()) {
		video.src = source;
	} else {
		const hls = new Hls();
		hls.loadSource(source);
		hls.attachMedia(video);
		window.hls = hls;


	}
	window.player = player;
}









