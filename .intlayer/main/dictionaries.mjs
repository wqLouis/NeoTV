import _19igtg5cmo5 from '../dictionary/common.json' with { type: 'json' };
import _1cwvm2e6qp2 from '../dictionary/home.json' with { type: 'json' };
import _49icimq6hh from '../dictionary/navigation.json' with { type: 'json' };
import _1m8nmr56ltl from '../dictionary/search.json' with { type: 'json' };
import _29q5aranknq from '../dictionary/settings.json' with { type: 'json' };

const dictionaries = {
  "common": _19igtg5cmo5,
  "home": _1cwvm2e6qp2,
  "navigation": _49icimq6hh,
  "search": _1m8nmr56ltl,
  "settings": _29q5aranknq
};
const getDictionaries = () => dictionaries;

export { getDictionaries };
export default dictionaries;
