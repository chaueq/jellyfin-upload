if(getApiKey() === null) {
    document.getElementById('access').classList.remove('hidden');
}
else {
    addCollections();
}


document.getElementById('apikey').addEventListener('keypress', (e) => {
    if(e.key === 'Enter') {
        const key = document.getElementById('apikey').value;
        if(key.length > 0) {
            localStorage.setItem('apikey', key);
            document.getElementById('access').classList.add('hidden');
            addCollections();
        }
    }
});

document.getElementById('collection').addEventListener('change', (e) => {
    updateSpace();
});

document.getElementById('file').addEventListener('change', (e) => {
    const name = document.getElementById('name');
    const text = document.querySelector('#upload>div.progress_text');
    setProgress(0);
    name.value = e.target.files[0].name;
    document.getElementById('file_label').innerText = e.target.files[0].name;
    text.innerText = 'UPLOAD';
});

document.getElementById('upload').addEventListener('click', async (e) => {
    const text = document.querySelector('#upload>div.progress_text');
    if(text.innerText != "UPLOAD" && text.innerText != "UPLOADING") {
        window.location.reload();
        return;
    }
    else if(text.innerText.includes('UPLOADING')) {
        return;
    }
    text.innerText = 'UPLOADING: 0%';

    const file = document.getElementById('file').files[0];
    const chunk_size = 10485760; //10 MB (used to be 100, then 90 but 10 is better for transfer stability)
    const chunks = Math.ceil(file.size/chunk_size);
    const max_attempts = 5;
    const interval = setInterval(updateSpace, 60000);
    
    for(let i = 0; i < chunks; ++i) {
        const offset = i * chunk_size;
        const data = file.slice(offset, offset + chunk_size);
        
        for(let attempt = 0; attempt < max_attempts; ++attempt) {
            try {
                const resp = await fetch(window.location.href, {
                    method: i == 0 ? 'PUT' : 'POST',
                    headers: {
                        'content-type': 'application/octet-stream',
                        'x-collection': document.getElementById('collection').value,
                        'x-filename': document.getElementById('name').value,
                        'x-apikey': getApiKey()
                    },
                    body: data
                });

                if(resp.ok) {
                    const done = (i+1) / chunks;
                    setProgress(done * 100).then(((done) => {
                        const text = document.querySelector('#upload>div.progress_text');
                        if(done == 1) {
                            text.innerText = 'DONE';
                            text.classList.add('success');
                        }
                    }).bind(null, done));
                    break;
                }
                else if(attempt == max_attempts-1) {
                    const text = document.querySelector('#upload>div.progress_text');
                    text.innerText = 'FAILED';
                    text.classList.add('fail');
                }
                else if(resp.status == 401 || resp.status == 403) {
                    document.getElementById('access').classList.remove('hidden');
                    return;
                }
                else {
                    await sleep(Math.pow(4, attempt) * 1000);
                }
            }
            catch(e) {
                await sleep(Math.pow(4, attempt) * 1000);
            }
        }
    }

    clearInterval(interval);
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
    const text = document.querySelector('#upload>div.progress_text');
    const progress = document.querySelector('div.progress_inner');
    const transitionEnd = whichTransitionEvent();
    const movement = new Promise((resolve) => {
        progress.addEventListener(transitionEnd, resolve, false);
    });
    if(percentage == 0) {
        text.innerText = 'UPLOAD';
        text.classList.remove('success', 'fail');
    }
    else {
        text.innerText = 'UPLOADING: ' + Math.round(percentage) + '%';
    }
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

function addCollections() {
    fetch('/collections', {
        headers: {
            'x-apikey': getApiKey()
        }
    }).then(async (resp) => {
        if(resp.ok) {
            const sel = document.getElementById('collection');
            const pairs = await resp.json();
            pairs.sort((a, b) => a[0].localeCompare(b[0]));
            for(pair of pairs) {
                let option = document.createElement('option');
                option.innerText = pair[0];
                option.value = pair[1];
                sel.appendChild(option);
            }
            updateSpace();
        }
        else if (resp.status == 403 || resp.status == 401) {
            document.getElementById('access').classList.remove('hidden');
        }
    });
}

function sleep(time) {
    return new Promise((resolve) => setTimeout(resolve, time));
}

async function updateSpace() {
    const space = document.getElementById('space')
    const resp = await fetch('/space', {
        method: 'GET',
        headers: {
            'x-apikey': getApiKey(),
            'x-collection': document.getElementById('collection').value
        }
    });

    if(resp.ok) {
        const bytes = parseInt(await resp.text());
        if (bytes < 1024) {
            space.innerText = bytes + ' B';
        }
        else if (bytes < 1048576) {
            space.innerText = Math.round(bytes / 1024) + ' kB';
        }
        else if (bytes < 1073741824) {
            space.innerText = Math.round(bytes / 1048576) + ' MB';
        }
        else if (bytes < 1099511627776) {
            space.innerText = Math.round(bytes / 1073741824) + ' GB';
        }
        else {
            space.innerText = Math.round(bytes / 1099511627776) + ' TB';
        }
    }
    else {
        space.innerText = 'Error';
    }
}