if(getApiKey() === null) {
    document.getElementById('access').classList.remove('hidden');
}

document.getElementById('apikey').addEventListener('keypress', (e) => {
    if(e.key === 'Enter') {
        const key = document.getElementById('apikey').value;
        if(key.length > 0) {
            localStorage.setItem('apikey', key);
            document.getElementById('access').classList.add('hidden');
        }
    }
});

document.getElementById('upload').addEventListener('click', (e) => {
    console.log('Sending file now...');
    const xhr = new XMLHttpRequest();

    xhr.onreadystatechange = function() {
        if (xhr.readyState == XMLHttpRequest.DONE) {
            window.location.reload();
        }
    }

    xhr.upload.onprogress = (e) => {
        console.log(e);
        const prcnt = (e.loaded / e.total) * 100;
        setProgress(prcnt);
    }

    xhr.open('POST', window.location.href);
    xhr.setRequestHeader("Content-Type", "application/octet-stream");
    xhr.setRequestHeader('x-folder', document.getElementById('folder').value);
    xhr.setRequestHeader('x-filename', document.getElementById('name').value);
    xhr.setRequestHeader('x-apikey', getApiKey());

    xhr.send(document.getElementById('file').files[0]);
})

function getApiKey() {
    const key = localStorage.getItem('apikey');
    if(key === null) {
        const input = document.getElementById('apikey');
        if(input.value.length > 0) {
            localStorage.setItem('apikey', input.value);
            return input.value;
        }
        return null;        
    }
    return key;
}

function setProgress(percentage) {
    const progress = document.querySelector('div.progress_inner');
    const transitionEnd = whichTransitionEvent();
    const movement = new Promise((resolve) => {
        progress.addEventListener(transitionEnd, resolve, false);
    });
    progress.style.width = percentage + '%';
    return movement;
}

function whichTransitionEvent(){
    var t;
    var el = document.createElement('fakeelement');
    var transitions = {
      'transition':'transitionend',
      'OTransition':'oTransitionEnd',
      'MozTransition':'transitionend',
      'WebkitTransition':'webkitTransitionEnd'
    }

    for(t in transitions){
        if( el.style[t] !== undefined ){
            return transitions[t];
        }
    }
}