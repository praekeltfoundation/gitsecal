# gitsecal
A little tool for finding github security alerts for all repos in an org.

NOTE: The github auth token used with this program must belong to a user with
sufficient permissions to see security alerts. I have only tried this as an org
owner, but in theory it should work fine for repo admins as well (although in
that case it will only return alerts for repos the user is an admin on).

## Prerequisites

You'll need to first acquire a copy of the github graphql schema with previews
enabled, which I didn't include because it's pretty huge. This requires an
authed HTTP request, which you can make as follows:

```
curl \
  -H "Accept: application/vnd.github.vixen-preview+json" \
  -H "Authorization: bearer $GH_OAUTH_TOKEN" \
  https://api.github.com/graphql \
  | jq '.data' > github-schema.json
```

The schema must be in a file named `github-schema.json` in the project root.

See
https://help.github.com/en/articles/creating-a-personal-access-token-for-the-command-line
for auth token stuff.

## Usage

Once you have the schema json, you're ready to run `gitsecal`. During dev, it's
easiest to do this with `cargo run`:

```
cargo run -- --oauth-token <token> --org <org>
```

All options can also be provided in environment variables (`GH_OAUTH_TOKEN`,
etc.), and I recommend using https://github.com/sorah/envchain or something
similar to avoid mucking about with credentials on the command line.

```
envchain gitsecal-p16n cargo run -- --org praekeltfoundation
```
