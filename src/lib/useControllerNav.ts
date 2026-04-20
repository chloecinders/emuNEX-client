import { onUnmounted, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useControllerStore } from "../stores/ControllerStore";
import { useGameStore } from "../stores/GameStore";

const FOCUSABLE_SEL =
    'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [role="switch"]:not([tabindex="-1"]), [tabindex]:not([tabindex="-1"])';
const CONTENT_SEL = ".c-shell__content";
const NAV_SEL = ".c-shell__top-nav-link";
const CTRL_CLASS = "is-controller-navigating";

function enterControllerMode() {
    document.body.classList.add(CTRL_CLASS);
}

function exitControllerMode() {
    document.body.classList.remove(CTRL_CLASS);
}

function getScrollContainer(): HTMLElement | null {
    return document.querySelector<HTMLElement>(CONTENT_SEL);
}

function isVisible(el: HTMLElement): boolean {
    return el.offsetParent !== null && getComputedStyle(el).visibility !== "hidden";
}

function getFocusableElements(): HTMLElement[] {
    const active = document.activeElement as HTMLElement | null;

    const openSelect = document.querySelector<HTMLElement>(".c-select__wrapper--open");
    if (openSelect) {
        return Array.from(openSelect.querySelectorAll<HTMLElement>(FOCUSABLE_SEL)).filter(isVisible);
    }

    const contextMenu = document.querySelector<HTMLElement>(".c-context-menu");
    if (contextMenu) {
        return Array.from(contextMenu.querySelectorAll<HTMLElement>(FOCUSABLE_SEL)).filter(isVisible);
    }

    const versionsMenu = document.querySelector<HTMLElement>(".c-bottom-panel__versions");
    if (versionsMenu) {
        return Array.from(versionsMenu.querySelectorAll<HTMLElement>(FOCUSABLE_SEL)).filter(isVisible);
    }

    const modal = document.querySelector<HTMLElement>(".c-modal__overlay");

    if (modal) {
        return Array.from(modal.querySelectorAll<HTMLElement>(FOCUSABLE_SEL)).filter(isVisible);
    }

    const panel = active?.closest<HTMLElement>(".c-bottom-panel");
    if (panel) {
        return Array.from(panel.querySelectorAll<HTMLElement>(FOCUSABLE_SEL)).filter(isVisible);
    }

    const shell = document.querySelector<HTMLElement>(".c-shell") ?? document.body;
    return Array.from(shell.querySelectorAll<HTMLElement>(FOCUSABLE_SEL))
        .filter(isVisible)
        .filter(el => {
            const inNav = el.closest(".c-shell__top-nav");
            const inPanel = el.closest(".c-bottom-panel");
            const inSearch = el.closest(".c-search");
            return !inNav && !inPanel && !inSearch;
        });
}

function getNavLinks(): HTMLElement[] {
    return Array.from(document.querySelectorAll<HTMLElement>(NAV_SEL));
}

function getOpenModal(): HTMLElement | null {
    return document.querySelector<HTMLElement>(".c-modal__overlay");
}

function closeModal(): boolean {
    const modal = getOpenModal();
    if (!modal) return false;
    const closeBtn = modal.querySelector<HTMLButtonElement>(".c-modal__close");
    closeBtn?.click();
    return true;
}

function isInBottomPanel(): boolean {
    return !!document.activeElement?.closest(".c-bottom-panel");
}

function syncSelection(el: HTMLElement) {
    if (el.classList.contains("c-game-card")) {
        const gameId = el.getAttribute("data-game-id");
        if (gameId) {
            const gameStore = useGameStore();
            gameStore.currentSelectedGame = gameId;
        }
    }
}

function getDistance(rect1: DOMRect, rect2: DOMRect, direction: string): number {
    const p1 = {
        x: rect1.left + rect1.width / 2,
        y: rect1.top + rect1.height / 2,
    };
    const p2 = {
        x: rect2.left + rect2.width / 2,
        y: rect2.top + rect2.height / 2,
    };

    const dx = p2.x - p1.x;
    const dy = p2.y - p1.y;

    if (direction === "up" || direction === "down") {
        return Math.sqrt((dx * dx * 0.15) + dy * dy);
    } else {
        return Math.sqrt(dx * dx + (dy * dy * 0.15));
    }
}

function moveFocus(direction: string, isRepeat = false) {
    const active = document.activeElement as HTMLElement | null;
    if (!active || active === document.body) {
        focusFirstInContent();
        return;
    }

    if (active instanceof HTMLInputElement && active.type === "range") {
        if (direction === "left" || direction === "right") {
            const turbo = isRepeat ? 10 : 1;
            for (let i = 0; i < turbo; i++) {
                if (direction === "left") active.stepDown();
                else active.stepUp();
            }

            active.dispatchEvent(new Event("input", { bubbles: true }));
            active.dispatchEvent(new Event("change", { bubbles: true }));
            return;
        }
    }

    const rect = active.getBoundingClientRect();
    const focusables = getFocusableElements();

    let filtered = focusables.filter((el) => {
        if (el === active) return false;
        const r = el.getBoundingClientRect();
        if (direction === "up") return r.bottom <= rect.top + 5;
        if (direction === "down") return r.top >= rect.bottom - 5;
        if (direction === "left") return r.right <= rect.left + 5;
        if (direction === "right") return r.left >= rect.right - 5;
        return false;
    });

    if (filtered.length === 0) {
        if (direction === "right") {
            filtered = focusables.filter((el) => el.getBoundingClientRect().top >= rect.bottom - 5);
        } else if (direction === "left") {
            filtered = focusables.filter((el) => el.getBoundingClientRect().bottom <= rect.top + 5);
        } else if (direction === "down") {
            filtered = focusables.filter((el) => el.getBoundingClientRect().top >= rect.bottom - 5);
        } else if (direction === "up") {
            filtered = focusables.filter((el) => el.getBoundingClientRect().bottom <= rect.top + 5);
        }
    }

    if (filtered.length === 0) return;

    const closest = filtered.reduce((prev, curr) => {
        const distPrev = getDistance(rect, prev.getBoundingClientRect(), direction);
        const distCurr = getDistance(rect, curr.getBoundingClientRect(), direction);
        return distCurr < distPrev ? curr : prev;
    });

    closest.focus();
    syncSelection(closest);
    enterControllerMode();
    closest.scrollIntoView({ block: "nearest", behavior: "smooth" });
}

function focusGameInfo() {
    const panel = document.querySelector<HTMLElement>(".c-bottom-panel");
    if (!panel) return false;

    const first = panel.querySelector<HTMLElement>(FOCUSABLE_SEL);
    if (first) {
        enterControllerMode();
        first.focus({ preventScroll: true });
        first.scrollIntoView({ block: "nearest", behavior: "smooth" });
        return true;
    }
    return false;
}

function openContextMenu() {
    const openMenu = document.querySelector<HTMLElement>(".c-context-menu");
    if (openMenu) {
        window.dispatchEvent(new CustomEvent("close-all-context-menus"));
        focusLastSelectedGame();
        return;
    }

    const el = document.activeElement as HTMLElement | null;
    if (!el || !el.classList.contains("c-game-card")) return;

    const rect = el.getBoundingClientRect();
    const event = new MouseEvent("contextmenu", {
        bubbles: true,
        cancelable: true,
        view: window,
        clientX: rect.left + rect.width / 2,
        clientY: rect.top + rect.height / 2,
    });
    el.dispatchEvent(event);
}

function focusSearch() {
    const search = document.querySelector<HTMLInputElement>(".c-search input");
    if (search) {
        enterControllerMode();
        search.focus();
    }
}

function focusLastSelectedGame() {
    const gameStore = useGameStore();
    const gid = gameStore.currentSelectedGame;
    if (!gid) return false;

    const card = document.querySelector<HTMLElement>(`.c-game-card[data-game-id="${gid}"]`);
    if (card && isVisible(card)) {
        enterControllerMode();
        card.focus({ preventScroll: true });
        card.scrollIntoView({ block: "nearest", behavior: "smooth" });
        return true;
    }
    return false;
}

function clickFocused() {
    const el = document.activeElement as HTMLElement | null;
    if (!el || el === document.body) return;

    if (el.classList.contains("c-game-card")) {
        const gameId = el.getAttribute("data-game-id");
        if (gameId) {
            window.dispatchEvent(new CustomEvent("request-play-game", { detail: { gameId } }));
            return;
        }
    }

    el.click();
}

function scrollContent(dir: "up" | "down") {
    const el = getScrollContainer();
    if (!el) return;
    const amount = el.clientHeight * 0.35;
    el.scrollBy({ top: dir === "down" ? amount : -amount, behavior: "smooth" });
}

function switchTab(dir: "prev" | "next") {
    const links = getNavLinks();
    if (!links.length) return;

    const activeIdx = links.findIndex((l) => l.classList.contains("router-link-active"));
    const idx = activeIdx === -1 ? 0 : activeIdx;

    const target = dir === "next"
        ? (links[idx + 1] ?? links[0])
        : (links[idx - 1] ?? links[links.length - 1]);

    enterControllerMode();
    target.click();
}

function focusFirstInContent() {
    setTimeout(() => {
        const fresh = getFocusableElements();
        if (!fresh.length) return;
        enterControllerMode();
        fresh[0].focus({ preventScroll: true });
        syncSelection(fresh[0]);
    }, 150);
}

const NAV_ACTIONS = {
    up: ["DPad-Up", "LS Up"],
    down: ["DPad-Down", "LS Down"],
    left: ["DPad-Left", "LS Left"],
    right: ["DPad-Right", "LS Right"],
    confirm: ["A"],
    back: ["B"],
    tabPrev: ["L"],
    tabNext: ["R"],
    scrollUp: ["RS Up"],
    scrollDown: ["RS Down"],
    home: ["Start"],
    info: ["X"],
    menu: ["Y"],
    search: ["Select"],
} as const;

type NavDir = keyof typeof NAV_ACTIONS;

const INITIAL_DELAY = 300;
const REPEAT_MS = 50;

export function useControllerNav() {
    const store = useControllerStore();
    const router = useRouter();
    const route = useRoute();

    const onMouseDown = () => exitControllerMode();
    document.addEventListener("mousedown", onMouseDown);

    const stopRouteWatch = watch(() => route.fullPath, () => {
        focusFirstInContent();
    });

    const prev: Record<NavDir, boolean> = {
        up: false, down: false, left: false, right: false,
        confirm: false, back: false, tabPrev: false, tabNext: false,
        scrollUp: false, scrollDown: false, home: false, info: false,
        menu: false, search: false,
    };

    const repeatTimers: Partial<Record<NavDir, ReturnType<typeof setTimeout>>> = {};
    const repeatIntervals: Partial<Record<NavDir, ReturnType<typeof setInterval>>> = {};

    function isPressed(dir: NavDir): boolean {
        return (NAV_ACTIONS[dir] as readonly string[]).some((a) => !!store.pressedActions[a]);
    }

    function act(dir: NavDir, isRepeat = false) {
        switch (dir) {
            case "up":
            case "down":
            case "left":
            case "right":
                moveFocus(dir, isRepeat);
                break;
            case "confirm":
                clickFocused();
                break;
            case "back": {
                const openSel = document.querySelector<HTMLElement>(".c-select__wrapper--open");
                if (openSel) {
                    openSel.click();
                    break;
                }

                const openMenu = document.querySelector<HTMLElement>(".c-context-menu");
                if (openMenu) {
                    window.dispatchEvent(new CustomEvent("close-all-context-menus"));
                    focusLastSelectedGame();
                    break;
                }

                const openVersions = document.querySelector<HTMLElement>(".c-bottom-panel__versions");
                if (openVersions) {
                    window.dispatchEvent(new CustomEvent("close-version-picker"));
                    focusLastSelectedGame();
                    break;
                }

                if (closeModal()) break;

                if (isInBottomPanel()) {
                    if (focusLastSelectedGame()) break;
                }

                break;
            }
            case "tabPrev":
                switchTab("prev");
                break;
            case "tabNext":
                switchTab("next");
                break;
            case "scrollUp":
                scrollContent("up");
                break;
            case "scrollDown":
                scrollContent("down");
                break;
            case "home":
                router.push("/");
                break;
            case "info":
                focusGameInfo();
                break;
            case "menu":
                openContextMenu();
                break;
            case "search":
                focusSearch();
                break;
        }
    }

    const REPEATABLE: NavDir[] = ["up", "down", "left", "right", "scrollUp", "scrollDown"];

    function stopRepeat(dir: NavDir) {
        if (repeatTimers[dir] != null) { clearTimeout(repeatTimers[dir]!); delete repeatTimers[dir]; }
        if (repeatIntervals[dir] != null) { clearInterval(repeatIntervals[dir]!); delete repeatIntervals[dir]; }
    }

    function startRepeat(dir: NavDir) {
        if (!REPEATABLE.includes(dir)) return;
        stopRepeat(dir);
        repeatTimers[dir] = setTimeout(() => {
            if (!isPressed(dir)) return;
            act(dir, true);
            repeatIntervals[dir] = setInterval(() => {
                if (!isPressed(dir)) { stopRepeat(dir); return; }
                act(dir, true);
            }, REPEAT_MS);
        }, INITIAL_DELAY);
    }

    let lastNavEnabled = true;
    let syncFrames = 0;
    let rafHandle: number | null = null;

    function tick() {
        const isEnabled = store.isNavEnabled;

        if (isEnabled) {
            if (!lastNavEnabled) {
                syncFrames = 10;
            }

            if (syncFrames > 0) {
                for (const dir of Object.keys(NAV_ACTIONS) as NavDir[]) {
                    prev[dir] = isPressed(dir);
                }
                syncFrames--;
            } else {
                for (const dir of Object.keys(NAV_ACTIONS) as NavDir[]) {
                    const now = isPressed(dir);
                    const was = prev[dir];

                    if (now && !was) {
                        act(dir);
                        startRepeat(dir);
                    } else if (!now && was) {
                        stopRepeat(dir);
                    }
                    prev[dir] = now;
                }
            }
        } else {
            const actions = store.pressedActions;
            if (actions["L"] && actions["R"] && actions["LS Click"] && actions["RS Click"]) {
                store.isNavEnabled = true;
            }

            for (const dir of Object.keys(NAV_ACTIONS) as NavDir[]) {
                if (prev[dir]) {
                    stopRepeat(dir);
                    prev[dir] = false;
                }
            }
        }

        lastNavEnabled = isEnabled;
        rafHandle = requestAnimationFrame(tick);
    }

    rafHandle = requestAnimationFrame(tick);

    onUnmounted(() => {
        stopRouteWatch();
        document.removeEventListener("mousedown", onMouseDown);
        exitControllerMode();
        if (rafHandle !== null) cancelAnimationFrame(rafHandle);
        for (const dir of Object.keys(NAV_ACTIONS) as NavDir[]) stopRepeat(dir);
    });
}

