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

document.getElementById('file').addEventListener('change', (e) => {
    const name = document.getElementById('name');
    if(name.value.length == 0) {
        name.value = e.target.files[0].name;
        document.getElementById('file_label').innerText = e.target.files[0].name;
    }
});

document.getElementById('upload').addEventListener('click', async (e) => {
    const text = document.querySelector('#upload>div.progress_text');
    if(text.innerText != "UPLOAD" && text.innerText != "UPLOADING") {
        window.location.reload();
        return;
    }
    else if(text.innerText == "UPLOADING") {
        return;
    }
    text.innerText = 'UPLOADING';

    const file = document.getElementById('file').files[0];
    const chunk_size = 94371840; //90 MB (used to be 100 but 90 is safer for free cloudflare limitations)
    const chunks = Math.ceil(file.size/chunk_size);
    
    for(let i = 0; i < chunks; ++i) {
        const offset = i * chunk_size;
        const data = file.slice(offset, offset + chunk_size);
        
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

                    if(document.querySelector('#upload>.progress_text').innerText == 'DONE') {
                        setTimeout(() => {window.location.reload()}, 10000);
                    }
                }
            }).bind(null, done));
        }
        else {
            const text = document.querySelector('#upload>div.progress_text');
            text.innerText = 'FAILED';
            text.classList.add('fail');
        }
    }
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

function addCollections() {
    fetch('/collections', {
        headers: {
            'x-apikey': getApiKey()
        }
    }).then(async (resp) => {
        if(resp.ok) {
            const sel = document.getElementById('collection');
            const pairs = await resp.json();
            for(pair of pairs) {
                let option = document.createElement('option');
                option.innerText = pair[0];
                option.value = pair[1];
                sel.appendChild(option);
            }
        }
    });
}