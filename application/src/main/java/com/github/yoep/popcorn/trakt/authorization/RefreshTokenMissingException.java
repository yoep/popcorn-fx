package com.github.yoep.popcorn.trakt.authorization;

public class RefreshTokenMissingException extends AccessTokenException {
    public RefreshTokenMissingException() {
        super("Refresh token is missing! Unable to refresh the access token.");
    }
}
