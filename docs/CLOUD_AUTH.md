# Cloud datastore upgrade path

GitHub Pages only serves static files, so VYRON 0.1.0 deliberately does not put a private database credential or password database in the browser bundle.

For global username uniqueness and cross-device login, use a managed authentication provider such as Supabase Auth or Firebase Authentication plus a profile table with a unique normalized username.

Recommended model:

- Auth provider owns password verification and session tokens.
- `profiles.username_normalized` has a database-level UNIQUE constraint.
- The browser stores only the provider session token.
- Workspace files are stored locally in 0.1.0 and can later be synchronized to object storage.
- Never store plaintext passwords in a public table or GitHub repository.

This separation is intentional: GitHub Pages can host the VYRON UI, while the authentication provider supplies the stateful server-side datastore.
