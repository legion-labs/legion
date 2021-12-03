const puppeteer = require('puppeteer');

(async () => {
  const browser = await puppeteer.launch({ headless: true });
  for( let i =0; i < 1000; ++i ){
    const page = await browser.newPage();
    await page.goto('http://localhost:3000/cumulative-call-graph?process=f3fd83a4-f37e-4a55-8866-1c4e23b690ce&begin=-306.42144790743623&end=15597.635978869874');
    await page.waitForSelector( '#funlist', {
      visible: true,
    });
  }
  // await page.screenshot({ path: 'example.png' });
  await browser.close();
})();
