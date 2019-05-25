# nano-ring-signatures

Generate and verify ring signatures with Nano private keys and addresses!

Uses Abe-Ohkubo-Suzuki ring signatures: https://paper.dropbox.com/doc/Ring-Signatures-Part-2-AOS-Rings-Zf8c3AL0mg9bPp05PeZl6

This is just for messages, not transactions.
It isn't integrated into the protocol in any way.

**This has not received formal review**. While after a quick code review,
I'm satisfied that it won't leak your private key, it still might
leak your identity if I've implemented it incorrectly.

## Generating a ring signature
```
$ cargo run generate
Private key:
Account or empty to finish: xrb_394adknux4zs4quaoi3gkp9mrdaio3eppy74fqbtdgbzif57ry4qqsy4gsii
Account or empty to finish: xrb_3d1itajgodn7rj3grp6djt1zjoarzom6unf3bp3zaguo3r61rqozuago88tz
Account or empty to finish: xrb_3o4s5caq1j6nzyutzre3pzbznshri5ebibmq9fm69mqgni3regbix346csju
Account or empty to finish: 
Message: Hello world!

Sorted accounts list:
xrb_394adknux4zs4quaoi3gkp9mrdaio3eppy74fqbtdgbzif57ry4qqsy4gsii
xrb_3d1itajgodn7rj3grp6djt1zjoarzom6unf3bp3zaguo3r61rqozuago88tz
xrb_3o4s5caq1j6nzyutzre3pzbznshri5ebibmq9fm69mqgni3regbix346csju

Signature:
c326ce7003601e3f264005d0b209312cf31efe82c2b1e81de4677f7bef565d0d89df71b6dd8961273507ecded5b1ebb54951c107f8fca8bca6f7b43977d5e1068276793967f0a629718965c5af18071f4c4d0a28fe15866d16df50ff51676e03ecb0292098af308501620b6054e974f74a07332c38e18fe42113f3a39d85f106
```
Note that you don't need to input the signing account since you
already inputted its private key, but I did anyways
so you don't know which address I signed this example with :)

## Verifying a ring signature
```
$ cargo run verify
Account or signature to finish: xrb_394adknux4zs4quaoi3gkp9mrdaio3eppy74fqbtdgbzif57ry4qqsy4gsii
Account or signature to finish: xrb_3d1itajgodn7rj3grp6djt1zjoarzom6unf3bp3zaguo3r61rqozuago88tz
Account or signature to finish: xrb_3o4s5caq1j6nzyutzre3pzbznshri5ebibmq9fm69mqgni3regbix346csju
Account or signature to finish: c326ce7003601e3f264005d0b209312cf31efe82c2b1e81de4677f7bef565d0d89df71b6dd8961273507ecded5b1ebb54951c107f8fca8bca6f7b43977d5e1068276793967f0a629718965c5af18071f4c4d0a28fe15866d16df50ff51676e03ecb0292098af308501620b6054e974f74a07332c38e18fe42113f3a39d85f106
Message: Hello world!
Ring signature is valid! :)
```
