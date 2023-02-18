package testing.java.com.github.yoep.popcorn.backend.settings.models;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

class TraktSettingsTest extends AbstractPropertyTest<TraktSettings> {
    public TraktSettingsTest() {
        super(TraktSettings.class);
    }

    @Test
    void testSetAccessToken_whenValueIsDifferent_shouldNotifyPropertyChanged() {
        var newValue = new OAuth2AccessTokenWrapper();

        settings.setAccessToken(newValue);

        assertEquals(TraktSettings.ACCESS_TOKEN_PROPERTY, invokedPropertyName());
        assertEquals(newValue, invokedNewValue());
    }
}