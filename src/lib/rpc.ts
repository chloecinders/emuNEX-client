import { Activity, ActivityStatusDisplayType, Assets, Timestamps } from "./activity";
import { useGameStore } from "../stores/GameStore";
import { useStoragePath } from "./http";
import { invoke } from "@tauri-apps/api/core";

export { Activity } from "./activity";

export class DiscordRPC {
    static startTimestamp: number = 0;
    static active: boolean = false;
    static get defaultActivity(): Activity {
        return new Activity()
            .setDetails("Browsing Library...")
            .setTimestamps(new Timestamps(this.startTimestamp))
            .setAssets(new Assets().setLargeImage("emunex_icon"));
    };

    static async start(app_id: string) {
        this.startTimestamp = Date.now();

        try {
            await invoke("spawn_drpc_thread", { id: app_id });
            this.active = true;
            await this.reset();
        } catch (e) {
            console.error("Unable to start Discord RPC", e)
        }
    }

    static async reset() {
        if (!this.active) {
            return;
        }

        try {
            await invoke("set_drpc_activity", { activity: this.defaultActivity });
        } catch (e) {
            console.error("Unable to reset activity to default", e);
        }
    }

    static async playGame(gameid: string) {
        if (!this.active) {
            return;
        }

        const gameStore = useGameStore();
        const game = await gameStore.fetchGame(gameid);

        if (game) {
            const gameIcon = useStoragePath(game.image_path.replace('/covers/', '/icons/').replace('.webp', '.png'));
            const activity = new Activity()
                .setDetails(`${game.title}`)
                .setTimestamps(new Timestamps(Date.now()))
                .setAssets(new Assets().setLargeImage(gameIcon).setSmallImage("emunex_icon"))
                .setDisplayType(ActivityStatusDisplayType.Details);

            await invoke("set_drpc_activity", { activity });
        }
    }
}

