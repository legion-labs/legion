import { Broadcast, MainExecutor, Mpsc, Subscription } from "./index";

export function runExample(mainExecutor: MainExecutor, execute: boolean) {
  if (!execute) {
    return () => {
      // NOOP
    };
  }

  const mpsc = new Mpsc<number>(mainExecutor);
  const broadcast = new Broadcast<number>(mainExecutor);

  const subscription = new Subscription(mpsc, (message) => {
    console.log("Got a message (mpsc): ", message);
  });

  const intervalId = setInterval(() => {
    subscription.send(Math.random());
  }, 1_000);

  const subscription2 = new Subscription(mpsc, (message) => {
    console.log("Got a message (mpsc): ", message);
  });

  const intervalId2 = setInterval(() => {
    subscription2.send(Math.random());
  }, 1_000);

  const subscription3 = new Subscription(broadcast, (message) => {
    console.log("Got a message (broadcast): ", message);
  });

  const intervalId3 = setInterval(() => {
    subscription3.send(Math.random());
  }, 1_000);

  const subscription4 = new Subscription(broadcast, (message) => {
    console.log("Got a message (broadcast): ", message);
  });

  const intervalId4 = setInterval(() => {
    subscription4.send(Math.random());
  }, 1_000);

  return () => {
    clearInterval(intervalId);
    clearInterval(intervalId2);
    clearInterval(intervalId3);
    clearInterval(intervalId4);
  };
}
