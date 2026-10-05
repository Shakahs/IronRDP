import type { UserInteraction } from './UserInteraction';

/**
 * The `detail` of the `ready` event `<iron-remote-desktop>` dispatches once its canvas is set up: the element's
 * `UserInteraction`, its one handoff to the host page.
 */
export interface ReadyEventDetail {
    readonly irgUserInteraction: UserInteraction;
}

/**
 * The `ready` event `<iron-remote-desktop>` dispatches once its canvas is set up, a bubbling `CustomEvent`.
 */
export interface ReadyEvent {
    readonly detail: ReadyEventDetail;
}
