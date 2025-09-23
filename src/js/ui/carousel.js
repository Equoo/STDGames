
export function init_carousel() {
	let active_index = 0;

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

        // Update layout on scroll to maintain proper sizing
        const carousel = document.getElementById('carousel');
        
 

        carousel.addEventListener('wheel', (e) => {
            e.preventDefault();
	    let list = carousel.children[0];
	    let old_active = active_index;
            active_index = (active_index + (e.deltaY > 0 ? 1 : -1)) % list.children.length;
	    active_index = active_index < 0 ? list.children.length - 1 : active_index;
	    carousel.scrollLeft = list.children[old_active].offsetLeft + list.children[active_index].offsetWidth - carousel.offsetWidth * 0.04 + 15 - carousel.offsetLeft;
	    list.children[active_index].style.width = 'calc(66% - 15px)';
	    list.children[old_active].style.width = 'calc(26% - 15px)';
        });
 

}
