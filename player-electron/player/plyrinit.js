//
// Hours spent on trying to make plyr's quality change api work with different hls sources -> 5h;
// You are free to append more hours
//



let params = new URLSearchParams(document.location.search);
const timings = params.get("timings");
const source = params.get("source");
const title = params.get("title");

document.addEventListener('DOMContentLoaded', () => {
	
	const video = document.querySelector('video');
	let titleBar = document.getElementById('titleText');
	titleBar.innerHTML = title;
	let parsedTimings;
	let skipPoints = null;
	try {
		parsedTimings = JSON.parse(timings);
		if (parsedTimings.length > 2) {
			skipPoints = [
				{ time: parsedTimings[0], label: 'Opening start' },
				{ time: parsedTimings[1], label: 'Opening end' },
				{ time: parsedTimings[2], label: 'Ending start' },
				{ time: parsedTimings[3], label: 'Ending End' },
			]
		}
		else {
			skipPoints = [
				{ time: parsedTimings[0], label: 'Opening start' },
				{ time: parsedTimings[1], label: 'Opening end' },
			]
		}
	
	}
	catch (e) {
		console.log("Skip points were not provided");
	}
	//
	

	//let inputParams

	const player = new Plyr(video, {
		markers: {
			enabled: true,
			points: skipPoints,
		},
		keyboard:{focused:true,global:true},
	});


	if (!Hls.isSupported()) {
		video.src = source;
	} else {
		const hls = new Hls();
		hls.loadSource(source);
		hls.attachMedia(video);
		window.hls = hls;
		
		
	}
	window.player = player;
});