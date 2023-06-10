
export async function get_pubkey() {
    return (await window.nostr.getPublicKey()).toString();
}
export async function encrypt_content(pubkey, content) {
    return (await window.nostr.nip04.encrypt(pubkey, content)).toString();
}
export async function sign_event(created_at, content, tags, pubkey) {

    console.log(created_at);

const event = {
    created_at: 1686368702,
    content: content,
    tags: tags,
    kind: 20420,
    pubkey: pubkey,
    
};

    console.log(event);
       let e = (await window.nostr.signEvent(event));

    return JSON.stringify(e);
}
