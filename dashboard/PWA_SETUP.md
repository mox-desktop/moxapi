# PWA Setup for Desktop Host Manager

## What's Been Added

Your Desktop Host Manager now has Progressive Web App (PWA) support with the following features:

### ✅ Completed
- **Web App Manifest** (`/static/manifest.json`) - Defines app metadata, icons, and display properties
- **Service Worker** (`/static/sw.js`) - Handles offline caching and background sync
- **PWA Meta Tags** - Added to `index.html` for proper mobile display
- **Service Worker Registration** - Automatically registers the service worker
- **Offline Page** (`/static/offline.html`) - Shown when user is offline
- **Placeholder Icons** - Basic icons for testing (need to be replaced)

### 🔧 PWA Features Included
- **Offline Support** - App works without internet connection
- **Install Prompt** - Users can install the app on their device
- **App-like Experience** - Runs in standalone mode without browser UI
- **Background Sync** - Handles offline actions when connection returns
- **Push Notifications** - Ready for future notification features

## Next Steps

### 1. Generate Proper Icons
The current icons are placeholders. You need to create proper PNG icons for all sizes:

**Required Sizes:**
- 72x72px
- 96x96px  
- 128x128px
- 144x144px
- 152x152px
- 192x192px
- 384x384px
- 512x512px

**Options for generating icons:**

**Option A: Use the provided SVG**
```bash
# Install ImageMagick if not available
nix-env -iA nixpkgs.imagemagick

# Generate icons from the SVG
cd dashboard/static/icons
for size in 72 96 128 144 152 192 384 512; do
  convert -background transparent -size ${size}x${size} ../icon.svg icon-${size}x${size}.png
done
```

**Option B: Use online tools**
- [PWA Builder](https://www.pwabuilder.com/imageGenerator)
- [Real Favicon Generator](https://realfavicongenerator.net/)
- [Favicon.io](https://favicon.io/)

**Option C: Use the SVG directly**
The `icon.svg` file can be used as a mask icon for Safari.

### 2. Add Screenshots (Optional)
Add screenshots to `/static/screenshots/` for better app store listings:
- `desktop.png` (1280x720) - Desktop view
- `mobile.png` (390x844) - Mobile view

### 3. Test PWA Features

**Test Installation:**
1. Open your app in Chrome/Edge
2. Look for the install button in the address bar
3. Or use the browser's "Install" menu option

**Test Offline Mode:**
1. Open your app
2. Disconnect from the internet
3. Refresh the page - it should still work
4. Try navigating to a new page - should show offline page

**Test Service Worker:**
1. Open Chrome DevTools
2. Go to Application tab
3. Check "Service Workers" section
4. Verify the service worker is registered and active

### 4. Customize for Your Needs

**Update Manifest:**
- Modify `name`, `short_name`, and `description` in `manifest.json`
- Adjust `theme_color` and `background_color` to match your brand
- Update `start_url` if your app has a different entry point

**Enhance Service Worker:**
- Add more resources to cache in `urlsToCache` array
- Implement custom offline strategies
- Add push notification handling

**Add Install Button (Optional):**
```javascript
// Add this to your main JavaScript
function showInstallButton() {
  if (deferredPrompt) {
    const installButton = document.createElement('button');
    installButton.textContent = 'Install App';
    installButton.onclick = () => {
      deferredPrompt.prompt();
      deferredPrompt.userChoice.then((choiceResult) => {
        if (choiceResult.outcome === 'accepted') {
          console.log('User accepted the install prompt');
        }
        deferredPrompt = null;
      });
    };
    document.body.appendChild(installButton);
  }
}
```

## PWA Requirements Checklist

- ✅ HTTPS (required for service workers)
- ✅ Web App Manifest
- ✅ Service Worker
- ✅ Responsive Design
- ✅ Fast Loading (< 3 seconds)
- ✅ Works Offline
- ⚠️ Proper Icons (need to generate)
- ⚠️ Screenshots (optional)

## Browser Support

- **Chrome/Edge**: Full PWA support
- **Firefox**: Full PWA support  
- **Safari**: Limited support (no service worker, but manifest works)
- **Mobile Browsers**: Full support on Android, limited on iOS

## Troubleshooting

**Service Worker Not Registering:**
- Ensure you're serving over HTTPS
- Check browser console for errors
- Verify the service worker file is accessible at `/static/sw.js`

**Install Prompt Not Showing:**
- App must meet PWA criteria (manifest, service worker, HTTPS)
- User must interact with the site for at least 2 minutes
- App must not already be installed

**Offline Not Working:**
- Check that resources are being cached
- Verify the service worker is active
- Test with browser's offline mode

## Additional Resources

- [MDN PWA Guide](https://developer.mozilla.org/en-US/docs/Web/Progressive_web_apps)
- [Web.dev PWA Guide](https://web.dev/progressive-web-apps/)
- [PWA Builder](https://www.pwabuilder.com/)
- [Lighthouse PWA Audit](https://developers.google.com/web/tools/lighthouse) 