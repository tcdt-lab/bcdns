async function resolveDomain() {
    const domain = document.getElementById('domainInput').value;
    if (!domain) return;

    try {
        const response = await fetch('/resolve/domain', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({ domain })
        });
        
        const result = await response.json();
        const resultElement = document.getElementById('domainResult');
        resultElement.style.display = 'block';
        resultElement.textContent = JSON.stringify(result, null, 2);
    } catch (error) {
        console.error('Error:', error);
    }
}

async function resolveAsset() {
    const asset = document.getElementById('assetInput').value;
    if (!asset) return;

    try {
        const response = await fetch('/resolve/asset', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({ asset })
        });
        
        const result = await response.json();
        const resultElement = document.getElementById('assetResult');
        resultElement.style.display = 'block';
        resultElement.textContent = JSON.stringify(result, null, 2);
    } catch (error) {
        console.error('Error:', error);
    }
}
