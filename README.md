# HTTP Proxy

This is a simple http proxy which support Block list / Allow List, it can be used as follow scenario:

1. With `-a` option, be used as a normal HTTP Proxy
2. With `-A path/to/allow/list` option, client can only access domains in the allow list.
3. With `-B path/to/block/list` option, clients' access to the domains in the block list will be banned.