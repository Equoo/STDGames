
function toggleVideo(button) {
            const videoItem = button.closest('.carousel-item');
            const video = videoItem.querySelector('video');
            
            if (video.paused) {
                // Pause all other videos
                document.querySelectorAll('video').forEach(v => {
                    if (v !== video) {
                        v.pause();
                        const otherButton = v.parentElement.querySelector('.play-button');
                        if (otherButton) updatePlayButton(otherButton, false);
                    }
                });
                
                video.play();
                updatePlayButton(button, true);
            } else {
                video.pause();
                updatePlayButton(button, false);
            }
        }

        function updatePlayButton(button, isPlaying) {
            if (!button) return;
            
            const icon = button.querySelector('.play-icon');
            if (isPlaying) {
                icon.innerHTML = '<rect x="6" y="4" width="4" height="16"></rect><rect x="14" y="4" width="4" height="16"></rect>';
            } else {
                icon.innerHTML = '<polygon points="5,3 19,12 5,21"></polygon>';
            }
        }


let active_index = 0;
let items = [];
let container;
let last_handle;
export function init_carousel(carousel) {
	container = carousel.children[0];
	items = Array.from(container.children);

	active_index = 0;
	container.insertBefore(items[items.length - 1], container.children[0]);
	carousel.scrollLeft = items[0].offsetLeft - carousel.offsetWidth * 0.04 - 15 - carousel.offsetLeft;
	items[active_index].style.width = 'calc(66% - 15px)';

	function handle(e) {
            e.preventDefault();

	    let old_active = active_index;

            active_index = (active_index + (e.deltaY > 0 ? 1 : -1)) % items.length;
	    active_index = active_index < 0 ? items.length - 1 : active_index;

	    if (e.deltaY > 0) {
	    	container.insertBefore(container.children[0], null);
	    	carousel.scrollLeft = items[old_active].offsetLeft + items[active_index].offsetWidth - carousel.offsetWidth * 0.04 + 15 - carousel.offsetLeft;
	    } else {	
	    	container.insertBefore(container.children[container.children.length - 1], container.children[0]);
	    	carousel.scrollLeft = items[active_index].offsetLeft - carousel.offsetLeft - carousel.offsetWidth * 0.04 - 15;
	    }
		
	    items[active_index].style.width = 'calc(66% - 15px)';
	    items[old_active].style.width = 'calc(26% - 15px)';

		if (items[active_index].className == "carousel-item video") {
			items[active_index].children[0].play();
		}
		if (items[old_active].className == "carousel-item video") {
			items[old_active].children[0].pause();
		}
	}
	carousel.removeEventListener('wheel', last_handle);
	carousel.addEventListener('wheel', handle);
	last_handle = handle;
}
