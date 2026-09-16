import { invoke } from "@tauri-apps/api/core";

const startButton =
    document.getElementById("startButton");

const status =
    document.getElementById("status");

const level =
    document.getElementById("level");

const dbDisplay =
    document.getElementById("db");

let running = false;

startButton.addEventListener(
    "click",
    async () => {

        if (!running) {
            await startMonitor();
        } else {
            await stopMonitor();
        }

    }
);


async function startMonitor() {

    try {

        status.textContent =
            "オーディオエンジン起動中...";

        await invoke("start_monitor");

        running = true;

        startButton.textContent =
            "停止";

        startButton.classList.add(
            "running"
        );

        status.textContent =
            "マイクモニター中";

    } catch (error) {

        console.error(error);

        status.textContent =
            `エラー: ${error}`;

    }
}


async function stopMonitor() {

    try {

        await invoke("stop_monitor");

        running = false;

        startButton.textContent =
            "マイク開始";

        startButton.classList.remove(
            "running"
        );

        status.textContent =
            "停止中";

        level.style.width =
            "0%";

        dbDisplay.textContent =
            "-∞ dBFS";

    } catch (error) {

        console.error(error);

        status.textContent =
            `エラー: ${error}`;

    }
}