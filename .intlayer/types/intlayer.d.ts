import "intlayer";
import _1voj9yywbqw from './common.ts';
import _balp9tlt6x from './home.ts';
import _10bbm9ynvd9 from './navigation.ts';
import _1eh04q93o15 from './search.ts';
import _2bwqxeemd4e from './settings.ts';

declare module 'intlayer' {
  interface __DictionaryRegistry {
    "common": typeof _1voj9yywbqw;
    "home": typeof _balp9tlt6x;
    "navigation": typeof _10bbm9ynvd9;
    "search": typeof _1eh04q93o15;
    "settings": typeof _2bwqxeemd4e;
  }

  interface __DeclaredLocalesRegistry {
    "en": 1;
    "zh-Hans": 1;
  }

  interface __RequiredLocalesRegistry {
    "en": 1;
    "zh-Hans": 1;
  }

  interface __SchemaRegistry {

  }

  interface __StrictModeRegistry { mode: 'inclusive' }

  interface __EditorRegistry { enabled : false } 
}
