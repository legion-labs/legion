const puppeteer = require('puppeteer');

(async () => {
  const browser = await puppeteer.launch({ headless: true });
  for( let i =0; i < 100; ++i ){
    const page = await browser.newPage();
    await page.goto('http://localhost:3000/cumulative-call-graph?process=894dfe0e-7339-40a5-a2ff-151afac061b9&begin=-1940.8247210417921&end=19287.176929494213');
    await page.waitForSelector( '#funlist', {
      visible: true,
    });
  }
  // await page.screenshot({ path: 'example.png' });
  await browser.close();
})();
