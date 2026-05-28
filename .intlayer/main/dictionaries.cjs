const _19igtg5cmo5 = require('../dictionary/common.json');
const _1cwvm2e6qp2 = require('../dictionary/home.json');
const _49icimq6hh = require('../dictionary/navigation.json');
const _1m8nmr56ltl = require('../dictionary/search.json');
const _29q5aranknq = require('../dictionary/settings.json');

const dictionaries = {
  "common": _19igtg5cmo5,
  "home": _1cwvm2e6qp2,
  "navigation": _49icimq6hh,
  "search": _1m8nmr56ltl,
  "settings": _29q5aranknq
};
const getDictionaries = () => dictionaries;

module.exports.getDictionaries = getDictionaries;
module.exports = dictionaries;
