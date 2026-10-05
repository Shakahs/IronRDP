export enum IronErrorKind {
    General = 0,
    WrongPassword = 1,
    LogonFailure = 2,
    AccessDenied = 3,
    RDCleanPath = 4,
    ProxyConnect = 5,
    NegotiationFailure = 6,
}

export interface RDCleanPathDetails {
    readonly httpStatusCode?: number;
    readonly wsaErrorCode?: number;
    readonly tlsAlertCode?: number;
}

export interface IronError {
    backtrace: () => string;
    kind: () => IronErrorKind;
    rdcleanpathDetails: () => RDCleanPathDetails | undefined;
}

/**
 * The `IronError` a failure is, if it is one: a session's promises reject with it, and TypeScript leaves a rejection
 * untyped. It is recognized by its `kind` and `backtrace` methods, which the backend's error and this package's own
 * both carry.
 */
export function ironError(failure: unknown): IronError | undefined {
    if (typeof failure !== 'object' || failure === null) {
        return undefined;
    }
    const candidate = failure as Partial<IronError>;
    return typeof candidate.kind === 'function' && typeof candidate.backtrace === 'function'
        ? (failure as IronError)
        : undefined;
}
