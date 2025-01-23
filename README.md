# Development

Your new jumpstart project includes basic organization with an organized `assets` folder and a `components` folder.
If you chose to develop with the router feature, you will also have a `views` folder.

### Serving Your App

Run the following command in the root of your project to start developing with the default platform:

## Develop

```bash
kill -9 $(lsof -ti:9090)
dx serve --platform desktop --port 9090 --features desktop
```

## Build

```bash
dx bundle --platform desktop --  --features desktop
```
